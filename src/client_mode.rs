use anyhow::{anyhow, Context};
use base64::Engine;
use clap::Subcommand;
use log::debug;
use std::io::{BufRead, BufReader, Read, Write};
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
    /// Press and release mouse buttons (`left`, `middle`, `right`).
    Click {
        #[arg(required = true, num_args = 1..)]
        buttons: Vec<String>,
    },
    /// Scroll vertically (REL_WHEEL).
    Wheel {
        /// Positive scrolls up, negative down.
        #[arg(allow_negative_numbers = true)]
        amount: i32,
    },
    /// Scroll horizontally (REL_HWHEEL).
    Hwheel {
        /// Positive scrolls right, negative left.
        #[arg(allow_negative_numbers = true)]
        amount: i32,
    },
    /// Move the pointer by X/Y units (REL_X / REL_Y).
    Mousemove {
        #[arg(allow_negative_numbers = true)]
        x: i32,
        #[arg(allow_negative_numbers = true)]
        y: i32,
    },
    /// Type text using the keyboard layout.
    Type {
        /// XKB layout name (default: $XKB_DEFAULT_LAYOUT, then `us`).
        #[arg(long, value_name = "LAYOUT")]
        layout: Option<String>,
        /// XKB variant name.
        #[arg(long, value_name = "VARIANT")]
        variant: Option<String>,
        /// How to produce characters: auto, layout, unicode or clipboard.
        #[arg(long, value_name = "METHOD")]
        method: Option<String>,
        /// Chord that pastes the clipboard for `--method clipboard`.
        #[arg(long, value_name = "CHORD")]
        paste: Option<String>,
        /// Restore the previous clipboard after `--method clipboard`.
        #[arg(long)]
        restore: bool,
        /// Pause after each chord in milliseconds (0-1000).
        #[arg(long, value_name = "MS")]
        delay: Option<u64>,
        /// Read the text from stdin instead of the arguments.
        #[arg(long, conflicts_with_all = ["text", "payload"])]
        stdin: bool,
        /// Base64-encoded text (no escaping needed).
        #[arg(long = "base64", value_name = "PAYLOAD", conflicts_with = "text")]
        payload: Option<String>,
        /// `TEXT`; words are joined with single spaces.
        #[arg(num_args = 0.., value_name = "TEXT")]
        text: Vec<String>,
    },
}

impl ClientCommand {
    /// Render this command as one protocol line. `type --stdin` reads stdin.
    fn protocol_line(&self) -> anyhow::Result<String> {
        match self {
            ClientCommand::Ping => Ok("ping".to_string()),
            ClientCommand::GetMode => Ok("get-mode".to_string()),
            ClientCommand::SetMode { mode } => Ok(format!("set-mode {mode}")),
            ClientCommand::Wheel { amount } => Ok(format!("wheel {amount}")),
            ClientCommand::Hwheel { amount } => Ok(format!("hwheel {amount}")),
            ClientCommand::Mousemove { x, y } => Ok(format!("mousemove {x} {y}")),
            ClientCommand::Click { buttons } => Ok(format!("click {}", buttons.join(" "))),
            ClientCommand::Type {
                layout,
                variant,
                method,
                paste,
                restore,
                delay,
                stdin,
                payload,
                text,
            } => render_type_command(
                layout.as_deref(),
                variant.as_deref(),
                method.as_deref(),
                paste.as_deref(),
                *restore,
                *delay,
                *stdin,
                payload.as_deref(),
                text,
            ),
            ClientCommand::Key { device, args }
            | ClientCommand::Keydown { device, args }
            | ClientCommand::Keyup { device, args } => {
                let verb = match self {
                    ClientCommand::Key { .. } => "key",
                    ClientCommand::Keydown { .. } => "keydown",
                    _ => "keyup",
                };
                Ok(render_key_command(verb, device, args))
            }
            ClientCommand::Sleep { milliseconds } => Ok(format!("sleep {milliseconds}")),
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
    let line = command.protocol_line()?;
    anyhow::ensure!(!line.contains(['\r', '\n']), "IPC command must be a single line (CR and LF are not allowed)");
    let path = socket
        .or_else(|| std::env::var("XREMAP_SOCKET").ok().map(PathBuf::from))
        .unwrap_or_else(crate::ipc_server::default_socket_path);

    let mut stream = UnixStream::connect(&path).with_context(|| {
        format!("cannot connect to xremap IPC socket {} (is `ipc.socket` enabled in the config?)", path.display())
    })?;

    let mut reader = BufReader::new(stream.try_clone()?);
    debug!("client: sending {:?} to {}", line, path.display());
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

/// Render a `type` protocol line. `--stdin` text is read here (client side),
/// normalised and base64-encoded so the protocol stays line-based.
fn render_type_command(
    layout: Option<&str>,
    variant: Option<&str>,
    method: Option<&str>,
    paste: Option<&str>,
    restore: bool,
    delay: Option<u64>,
    from_stdin: bool,
    payload: Option<&str>,
    text: &[String],
) -> anyhow::Result<String> {
    let mut line = String::from("type");
    if let Some(layout) = layout {
        line.push_str(" --layout ");
        line.push_str(layout);
    }
    if let Some(variant) = variant {
        line.push_str(" --variant ");
        line.push_str(variant);
    }
    if let Some(method) = method {
        line.push_str(" --method ");
        line.push_str(method);
    }
    if let Some(paste) = paste {
        line.push_str(" --paste ");
        line.push_str(paste);
    }
    if restore {
        line.push_str(" --restore");
    }
    if let Some(delay) = delay {
        line.push_str(" --delay ");
        line.push_str(&delay.to_string());
    }
    if let Some(payload) = payload {
        line.push_str(" --base64 ");
        line.push_str(payload);
        return Ok(line);
    }
    if from_stdin {
        let text = normalize_newlines(&read_stdin_text()?);
        line.push_str(" --base64 ");
        line.push_str(&base64::engine::general_purpose::STANDARD.encode(text.as_bytes()));
        return Ok(line);
    }
    if text.is_empty() {
        anyhow::bail!("missing TEXT (or use --stdin / --base64)");
    }
    // `--` keeps text that starts with a dash literal.
    line.push_str(" -- ");
    line.push_str(&text.join(" "));
    Ok(line)
}

fn read_stdin_text() -> anyhow::Result<String> {
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes)?;
    String::from_utf8(bytes).map_err(|_| anyhow!("stdin is not valid UTF-8"))
}

/// CRLF and lone CR become LF: xremap can inject Enter but has no CR key.
fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_without_stdin_is_rendered_verbatim() {
        let cmd = ClientCommand::Type {
            layout: Some("de".into()),
            variant: None,
            method: None,
            paste: None,
            restore: false,
            delay: None,
            stdin: false,
            payload: None,
            text: vec!["Grüße".into()],
        };
        assert_eq!(cmd.protocol_line().unwrap(), "type --layout de -- Grüße");
    }

    #[test]
    fn type_base64_payload_passes_through() {
        let cmd = ClientCommand::Type {
            layout: None,
            variant: Some("nodeadkeys".into()),
            method: None,
            paste: None,
            restore: false,
            delay: None,
            stdin: false,
            payload: Some("aGVsbG8=".into()),
            text: Vec::new(),
        };
        assert_eq!(cmd.protocol_line().unwrap(), "type --variant nodeadkeys --base64 aGVsbG8=");
    }

    #[test]
    fn type_requires_some_text() {
        let cmd = ClientCommand::Type {
            layout: None,
            variant: None,
            method: None,
            paste: None,
            restore: false,
            delay: None,
            stdin: false,
            payload: None,
            text: Vec::new(),
        };
        assert!(cmd.protocol_line().is_err());
    }

    #[test]
    fn type_clipboard_options_are_rendered() {
        let cmd = ClientCommand::Type {
            layout: None,
            variant: None,
            method: Some("clipboard".into()),
            paste: Some("ctrl+v".into()),
            restore: true,
            delay: None,
            stdin: false,
            payload: None,
            text: vec!["中文".into()],
        };
        assert_eq!(cmd.protocol_line().unwrap(), "type --method clipboard --paste ctrl+v --restore -- 中文");
    }

    #[test]
    fn newlines_are_normalised_to_lf() {
        assert_eq!(normalize_newlines("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert_eq!(normalize_newlines("no breaks"), "no breaks");
        assert_eq!(normalize_newlines("\r\n\r\n"), "\n\n");
    }

    #[test]
    fn plain_commands_are_rendered() {
        assert_eq!(ClientCommand::Ping.protocol_line().unwrap(), "ping");
        assert_eq!(ClientCommand::GetMode.protocol_line().unwrap(), "get-mode");
        assert_eq!(
            ClientCommand::SetMode { mode: "normal".into() }
                .protocol_line()
                .unwrap(),
            "set-mode normal"
        );
        assert_eq!(
            ClientCommand::Key {
                device: Some("kbd".into()),
                args: vec!["raw".into(), "ctrl+a".into()]
            }
            .protocol_line()
            .unwrap(),
            "key --device kbd raw ctrl+a"
        );
        assert_eq!(
            ClientCommand::Click {
                buttons: vec!["left".into(), "right".into()]
            }
            .protocol_line()
            .unwrap(),
            "click left right"
        );
        assert_eq!(ClientCommand::Mousemove { x: 4, y: -4 }.protocol_line().unwrap(), "mousemove 4 -4");
    }
}
