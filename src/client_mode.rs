use anyhow::{anyhow, Context};
use base64::Engine;
use std::io::{BufRead, BufReader, Read, Write};
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
        format!("cannot connect to xremap IPC socket {} (is `ipc.socket` enabled in the config?)", path.display())
    })?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut failed = false;
    if commands.is_empty() {
        // Stream commands from stdin over a single connection. stdin is the
        // command channel here, so `type --stdin` has no text to read.
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if reads_stdin(&line) {
                anyhow::bail!("`type --stdin` cannot be combined with `--client -`: stdin already carries commands");
            }
            failed |= !send_command(&mut stream, &mut reader, &line)?;
        }
    } else {
        for command in commands {
            let command = expand_text_source(command)?;
            failed |= !send_command(&mut stream, &mut reader, &command)?;
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

/// Does this command read its text from stdin? Only the leading options are
/// inspected, so `type hello --stdin` still types the literal text.
fn reads_stdin(command: &str) -> bool {
    let Some(rest) = command.strip_prefix("type ") else {
        return false;
    };
    let mut tokens = rest.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            "--stdin" => return true,
            "--layout" | "--variant" | "--paste" | "--method" | "--file" => {
                tokens.next();
            }
            "--restore" | "--base64" => {}
            _ => return false,
        }
    }
    false
}

/// Where the text of a `type --stdin` / `type --file PATH` command comes from.
enum TextSource {
    Stdin,
    File(PathBuf),
}

/// Expand `type --stdin` / `type --file PATH` into `type ... --base64 PAYLOAD`.
///
/// Reading the text on the client keeps the protocol line-based: base64 has no
/// line breaks, so multi-line text needs no escaping and the server needs no
/// new framing. CRLF and lone CR are normalised to LF because xremap cannot
/// inject CR. Commands that do not read a text source are returned unchanged.
fn expand_text_source(command: &str) -> anyhow::Result<String> {
    let Some(rest) = command.strip_prefix("type ") else {
        return Ok(command.to_string());
    };

    let mut options: Vec<String> = Vec::new();
    let mut source: Option<TextSource> = None;
    let mut inline_text: Option<String> = None;
    let mut tokens = rest.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            "--stdin" => source = Some(TextSource::Stdin),
            "--file" => {
                let path = tokens.next().ok_or_else(|| anyhow!("--file requires a path"))?;
                source = Some(TextSource::File(PathBuf::from(path)));
            }
            "--layout" | "--variant" | "--paste" | "--method" => {
                let value = tokens.next().ok_or_else(|| anyhow!("{token} requires a value"))?;
                options.push(token.to_string());
                options.push(value.to_string());
            }
            "--restore" | "--base64" => options.push(token.to_string()),
            "--" => {
                inline_text = Some(tokens.collect::<Vec<_>>().join(" "));
                break;
            }
            other => {
                inline_text = Some(std::iter::once(other).chain(tokens).collect::<Vec<_>>().join(" "));
                break;
            }
        }
    }

    let Some(source) = source else {
        return Ok(command.to_string());
    };
    if inline_text.is_some() {
        anyhow::bail!("`type --stdin`/`--file` cannot be combined with inline text");
    }

    let text = match source {
        TextSource::Stdin => read_stdin_text()?,
        TextSource::File(path) => {
            let text = std::fs::read(&path).with_context(|| format!("failed to read --file {}", path.display()))?;
            String::from_utf8(text).map_err(|_| anyhow!("--file {} is not valid UTF-8", path.display()))?
        }
    };
    let text = normalize_newlines(&text);
    let payload = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());

    let mut expanded = String::from("type");
    for option in options {
        expanded.push(' ');
        expanded.push_str(&option);
    }
    expanded.push_str(" --base64 ");
    expanded.push_str(&payload);
    Ok(expanded)
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

    fn expand_without_io(command: &str) -> String {
        // Only used for commands that must not touch stdin/files.
        expand_text_source(command).unwrap()
    }

    #[test]
    fn commands_without_a_text_source_are_unchanged() {
        for command in [
            "ping",
            "type hello world",
            "type --method clipboard --restore 中文",
            "type -- --stdin", // `--stdin` after `--` is literal text
            "typex --stdin",
            "key raw a",
        ] {
            assert_eq!(expand_without_io(command), command);
        }
    }

    #[test]
    fn newlines_are_normalised_to_lf() {
        assert_eq!(normalize_newlines("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert_eq!(normalize_newlines("no breaks"), "no breaks");
        assert_eq!(normalize_newlines("\r\n\r\n"), "\n\n");
    }

    #[test]
    fn file_source_becomes_base64_and_keeps_options() {
        let dir = std::env::temp_dir().join(format!("xremap-file-{}-{:x}", std::process::id(), fastrand::u64(..)));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("text.txt");
        std::fs::write(&path, "line1\r\nline2\n\ttab\r").unwrap();
        let command = expand_text_source(&format!("type --file {} --layout de --restore", path.display())).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        let payload = command
            .strip_prefix("type --layout de --restore --base64 ")
            .unwrap_or_else(|| panic!("unexpected expansion: {command}"));
        let decoded = base64::engine::general_purpose::STANDARD.decode(payload).unwrap();
        // CRLF and lone CR became LF; tab stays literal.
        assert_eq!(String::from_utf8(decoded).unwrap(), "line1\nline2\n\ttab\n");
    }

    #[test]
    fn file_source_rejects_missing_files_and_inline_text() {
        assert!(expand_text_source("type --file /nonexistent/xremap-review").is_err());
        assert!(expand_text_source("type --file /etc/hostname extra").is_err());
        assert!(expand_text_source("type --file").is_err());
        assert!(expand_text_source("type --stdin extra").is_err());
    }

    #[test]
    fn stdin_is_detected_only_for_type_commands() {
        assert!(reads_stdin("type --stdin"));
        assert!(reads_stdin("type --layout de --stdin"));
        assert!(!reads_stdin("type hello --stdin"));
        assert!(!reads_stdin("key raw a"));
    }
}
