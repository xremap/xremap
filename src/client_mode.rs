use anyhow::Context;
use clap::Subcommand;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Client-side IPC subcommands (`xremap <command>`). Each maps to one line of
/// the socket protocol; the server replies once per command.
#[derive(Subcommand, Debug, PartialEq)]
pub enum ClientCommand {
    /// Liveness check.
    Ping,
}

impl ClientCommand {
    /// Render this command as one protocol line.
    fn protocol_line(&self) -> String {
        match self {
            ClientCommand::Ping => "ping".to_string(),
        }
    }
}

/// Send one subcommand to a running xremap and print the reply. Exit code is
/// non-zero when the command fails.
pub fn run(command: ClientCommand, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let path = socket
        .or_else(|| std::env::var("XREMAP_SOCKET").ok().map(PathBuf::from))
        .unwrap_or_else(crate::ipc_server::default_socket_path);

    let mut stream = UnixStream::connect(&path).with_context(|| {
        format!("cannot connect to xremap IPC socket {} (is `ipc.socket` enabled in the config?)", path.display())
    })?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let line = command.protocol_line();
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reply = String::new();
    reader.read_line(&mut reply)?;
    if reply.is_empty() {
        anyhow::bail!("xremap closed the connection before replying");
    }
    print!("{reply}");
    std::io::stdout().flush()?;
    // Replies are "pong" or "err REASON". Any reply that is not an error
    // succeeded.
    if reply.starts_with("err") {
        anyhow::bail!("the command failed");
    }
    Ok(())
}
