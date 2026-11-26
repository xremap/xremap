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
    /// Inject key chords, each a press and a release. Requires `ipc.inject`.
    Key {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        #[serde(skip_serializing_if = "Option::is_none")]
        device: Option<String>,
        /// Pipeline layer the chords enter at: raw (default), keymap, modmap, expmap.
        #[arg(long, value_name = "SCOPE")]
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<crate::ipc_server::InjectScope>,
        /// Chords like `ctrl+a`, `f13` or `Down*3` (repeat suffix).
        #[arg(value_name = "CHORD", required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        chords: Vec<String>,
    },
    /// Press and hold the given chords. Requires `ipc.inject`.
    Keydown {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        #[serde(skip_serializing_if = "Option::is_none")]
        device: Option<String>,
        /// Pipeline layer the chords enter at: raw (default), keymap, modmap, expmap.
        #[arg(long, value_name = "SCOPE")]
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<crate::ipc_server::InjectScope>,
        /// Chords like `ctrl+a`, `f13` or `Down*3` (repeat suffix).
        #[arg(value_name = "CHORD", required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        chords: Vec<String>,
    },
    /// Release the given chords. Requires `ipc.inject`.
    Keyup {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        #[serde(skip_serializing_if = "Option::is_none")]
        device: Option<String>,
        /// Pipeline layer the chords enter at: raw (default), keymap, modmap, expmap.
        #[arg(long, value_name = "SCOPE")]
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<crate::ipc_server::InjectScope>,
        /// Chords like `ctrl+a`, `f13` or `Down*3` (repeat suffix).
        #[arg(value_name = "CHORD", required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        chords: Vec<String>,
    },
    /// Pause command processing (1-5000 ms). Blocks xremap's event loop. Requires `ipc.inject`.
    Sleep {
        /// Milliseconds to pause.
        milliseconds: u64,
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
    // Replies are "ok", "pong", the bare mode name of get-mode, or
    // "err REASON". Any reply that is not an error succeeded.
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
        assert_eq!(
            serde_json::to_string(&ClientCommand::Key {
                device: None,
                scope: None,
                chords: vec!["ctrl+a".to_string(), "Down*3".to_string()]
            })
            .unwrap(),
            r#"{"command":"key","chords":["ctrl+a","Down*3"]}"#
        );
        assert_eq!(
            serde_json::to_string(&ClientCommand::Key {
                device: Some("my-kbd".to_string()),
                scope: Some(crate::ipc_server::InjectScope::Remap),
                chords: vec!["f13".to_string()]
            })
            .unwrap(),
            r#"{"command":"key","device":"my-kbd","scope":"remap","chords":["f13"]}"#
        );
        assert_eq!(
            serde_json::to_string(&ClientCommand::Sleep { milliseconds: 100 }).unwrap(),
            r#"{"command":"sleep","milliseconds":100}"#
        );
    }
}
