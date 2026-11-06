use anyhow::Context;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// `xremap --client ...` entry point. Sends line-based commands to a running
/// xremap IPC socket and prints the replies. Exit code is non-zero when any
/// command fails.
pub fn run(commands: &[String], socket: Option<PathBuf>) -> anyhow::Result<()> {
    let path = socket
        .or_else(|| std::env::var("XREMAP_SOCKET").ok().map(PathBuf::from))
        .unwrap_or_else(crate::ipc_server::default_socket_path);

    let mut stream = UnixStream::connect(&path).with_context(|| {
        format!(
            "cannot connect to xremap IPC socket {} (is `ipc.socket` enabled in the config?)",
            path.display()
        )
    })?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut failed = false;
    if commands.is_empty() {
        // Stream commands from stdin over a single connection.
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            failed |= !send_command(&mut stream, &mut reader, &line)?;
        }
    } else {
        for command in commands {
            failed |= !send_command(&mut stream, &mut reader, command)?;
        }
    }

    if failed {
        anyhow::bail!("a command failed");
    }
    Ok(())
}

/// Returns true when the reply was a success.
fn send_command(stream: &mut UnixStream, reader: &mut BufReader<UnixStream>, command: &str) -> anyhow::Result<bool> {
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reply = String::new();
    reader.read_line(&mut reply)?;
    if reply.is_empty() {
        anyhow::bail!("xremap closed the connection before replying");
    }
    print!("{reply}");
    std::io::stdout().flush()?;
    // Replies are "ok", "pong", the bare mode name of get-mode, or
    // "err REASON". Any reply that is not an error succeeded.
    Ok(!reply.starts_with("err"))
}
