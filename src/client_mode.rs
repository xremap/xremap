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
    /// Query the active keymap mode.
    GetMode,
    /// Switch the active keymap mode.
    SetMode {
        /// Mode name to switch to.
        mode: String,
    },
    /// Inject key chords, each a press and a release.
    Key {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        device: Option<String>,
        /// `[SCOPE] CHORD...`, e.g. `raw ctrl+a f13` or `Down*3`.
        #[arg(required = true, num_args = 1..)]
        args: Vec<String>,
    },
    /// Press and hold the given chords.
    Keydown {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        device: Option<String>,
        /// `[SCOPE] CHORD...`, e.g. `raw ctrl+a` or `ctrl+a`.
        #[arg(required = true, num_args = 1..)]
        args: Vec<String>,
    },
    /// Release the given chords.
    Keyup {
        /// Simulated source device name (matches `device:` keymap conditions).
        #[arg(long, value_name = "NAME")]
        device: Option<String>,
        /// `[SCOPE] CHORD...`, e.g. `raw ctrl+a` or `ctrl+a`.
        #[arg(required = true, num_args = 1..)]
        args: Vec<String>,
    },
    /// Pause command processing (1-5000 ms). Blocks xremap's event loop.
    Sleep {
        /// Milliseconds to pause.
        milliseconds: u64,
    },
}

impl ClientCommand {
    /// Render this command as one protocol line.
    fn protocol_line(&self) -> String {
        match self {
            ClientCommand::Ping => "ping".to_string(),
            ClientCommand::GetMode => "get-mode".to_string(),
            ClientCommand::SetMode { mode } => format!("set-mode {mode}"),
            ClientCommand::Key { device, args }
            | ClientCommand::Keydown { device, args }
            | ClientCommand::Keyup { device, args } => {
                let verb = match self {
                    ClientCommand::Key { .. } => "key",
                    ClientCommand::Keydown { .. } => "keydown",
                    _ => "keyup",
                };
                render_key_command(verb, device, args)
            }
            ClientCommand::Sleep { milliseconds } => format!("sleep {milliseconds}"),
        }
    }
}

/// Build a `key`/`keydown`/`keyup` protocol line. The scope (if any) is the
/// first positional argument; the server parses it.
fn render_key_command(verb: &str, device: &Option<String>, args: &[String]) -> String {
    let mut line = verb.to_string();
    if let Some(device) = device {
        line.push_str(" --device ");
        line.push_str(device);
    }
    for arg in args {
        line.push(' ');
        line.push_str(arg);
    }
    line
}

/// Send one subcommand to a running xremap and print the reply. Exit code is
/// non-zero when the command fails.
pub fn run(command: ClientCommand, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let line = command.protocol_line();
    anyhow::ensure!(!line.contains(['\r', '\n']), "IPC command must be a single line (CR and LF are not allowed)");
    let path = socket
        .or_else(|| std::env::var("XREMAP_SOCKET").ok().map(PathBuf::from))
        .unwrap_or_else(crate::ipc_server::default_socket_path);

    let mut stream = UnixStream::connect(&path).with_context(|| {
        format!("cannot connect to xremap IPC socket {} (is `ipc.socket` enabled in the config?)", path.display())
    })?;

    let mut reader = BufReader::new(stream.try_clone()?);
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
    // Replies are "ok", "pong", the bare mode name of get-mode, or
    // "err REASON". Any reply that is not an error succeeded.
    if reply.starts_with("err") {
        anyhow::bail!("the command failed");
    }
    Ok(())
}
