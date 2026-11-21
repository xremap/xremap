use anyhow::Context;
use clap::Subcommand;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Client-side IPC subcommands (`xremap <command>`). Each serializes to one
/// JSON request line of the socket protocol; the server replies once per
/// command. Variant and field names are the wire format.
#[derive(Subcommand, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "command", rename_all = "kebab-case")]
pub enum ClientCommand {
    /// Liveness check.
    Ping,
    /// Query the active keymap mode.
    GetMode,
    /// Switch the active keymap mode.
    SetMode {
        /// Mode name to switch to.
        mode: String,
    },
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

    let line = serde_json::to_string(&command)?;
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_line_matches_the_server_protocol() {
        assert_eq!(serde_json::to_string(&ClientCommand::Ping).unwrap(), r#"{"command":"ping"}"#);
        assert_eq!(serde_json::to_string(&ClientCommand::GetMode).unwrap(), r#"{"command":"get-mode"}"#);
        assert_eq!(
            serde_json::to_string(&ClientCommand::SetMode {
                mode: "normal".to_string()
            })
            .unwrap(),
            r#"{"command":"set-mode","mode":"normal"}"#
        );
    }
}
