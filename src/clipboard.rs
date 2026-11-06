//! Text clipboard access via fixed system helpers. Both pipe I/O and child
//! completion are bounded by the same deadline; no reader/writer threads can
//! be stranded by an inherited pipe or a helper that stops consuming stdin.

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Writer {
    program: &'static str,
    args: &'static [&'static str],
}

impl Writer {
    fn run(&self, text: &str) -> Result<(), String> {
        run_helper(self.program, self.args, Some(text.as_bytes()), Duration::from_millis(WRITE_TIMEOUT_MS))
            .map(|_| ())
            .map_err(|err| format!("{}: {err}", self.program))
    }
}

/// Reap a failed/timed-out helper on every exit path. Successful clipboard
/// owners may have forked; only the direct helper process belongs to us.
struct Helper {
    child: Child,
    reaped: bool,
}

impl Drop for Helper {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn nonblocking(fd: &impl AsFd) -> io::Result<()> {
    let flags = OFlag::from_bits_truncate(fcntl(fd, FcntlArg::F_GETFL)?);
    fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;
    Ok(())
}

/// Transfer input/output while polling child completion. Check the deadline
/// even when a pipe keeps making progress, and bound output allocation too.
fn run_helper(program: &str, args: &[&str], input: Option<&[u8]>, timeout: Duration) -> io::Result<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    let child = Command::new(program)
        .args(args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(if input.is_none() { Stdio::piped() } else { Stdio::null() })
        .stderr(Stdio::null())
        .spawn()?;
    let mut helper = Helper { child, reaped: false };
    let mut stdin = helper.child.stdin.take();
    let mut stdout = helper.child.stdout.take();
    if let Some(pipe) = &stdin {
        nonblocking(pipe)?;
    }
    if let Some(pipe) = &stdout {
        nonblocking(pipe)?;
    }
    let input = input.unwrap_or_default();
    let mut written = 0;
    let mut output = Vec::new();
    loop {
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "clipboard helper I/O timed out"));
        }
        let mut progressed = false;
        if written == input.len() {
            // EOF is required for clipboard writers to finish reading.
            stdin.take();
        }
        if let Some(pipe) = &mut stdin {
            let end = input.len().min(written + 8192);
            match pipe.write(&input[written..end]) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "clipboard input pipe closed")),
                Ok(n) => {
                    written += n;
                    progressed = true;
                }
                Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) => {}
                Err(err) => return Err(err),
            }
            if written == input.len() {
                stdin.take();
            }
        }
        if let Some(pipe) = &mut stdout {
            let mut buffer = [0u8; 8192];
            match pipe.read(&mut buffer) {
                Ok(0) => {
                    stdout.take();
                    progressed = true;
                }
                Ok(n) => {
                    if output.len() + n > MAX_READ_BYTES {
                        return Err(io::Error::other("clipboard output exceeds 16 MiB"));
                    }
                    output.extend_from_slice(&buffer[..n]);
                    progressed = true;
                }
                Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) => {}
                Err(err) => return Err(err),
            }
        }
        if !helper.reaped {
            if let Some(status) = helper.child.try_wait()? {
                helper.reaped = true;
                if !status.success() {
                    return Err(io::Error::other(format!("clipboard helper exited with {status}")));
                }
            }
        }
        if helper.reaped && stdin.is_none() && stdout.is_none() {
            return Ok(output);
        }
        if !progressed {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

const WL_COPY: Writer = Writer {
    program: "wl-copy",
    args: &[],
};
const XSEL: Writer = Writer {
    program: "xsel",
    args: &["--clipboard", "--input"],
};
const XCLIP: Writer = Writer {
    program: "xclip",
    args: &["-selection", "clipboard"],
};

fn candidates() -> [&'static Writer; 3] {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        [&WL_COPY, &XSEL, &XCLIP]
    } else {
        [&XSEL, &XCLIP, &WL_COPY]
    }
}

fn run_read(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    String::from_utf8(run_helper(program, args, None, timeout).ok()?).ok()
}

/// Snapshot only text, up to 16 MiB, not image/other clipboard MIME types.
pub fn read() -> Option<String> {
    let timeout = Duration::from_millis(READ_TIMEOUT_MS);
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        run_read("wl-paste", &["--no-newline"], timeout)
            .or_else(|| run_read("xsel", &["--clipboard", "--output"], timeout))
            .or_else(|| run_read("xclip", &["-selection", "clipboard", "-o"], timeout))
    } else {
        run_read("xsel", &["--clipboard", "--output"], timeout)
            .or_else(|| run_read("xclip", &["-selection", "clipboard", "-o"], timeout))
            .or_else(|| run_read("wl-paste", &["--no-newline"], timeout))
    }
}

/// Best-effort delayed text restore. The delay is not an application ACK and
/// may race with a late paste or a user changing the clipboard in between.
pub fn restore_later(text: String, delay: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        if let Err(err) = write(&text) {
            eprintln!("xremap clipboard: restore failed: {err}");
        }
    });
}

/// Write through fixed helpers only; no user-supplied program/arguments.
pub fn write(text: &str) -> Result<(), String> {
    let mut errors = Vec::new();
    for writer in candidates() {
        match writer.run(text) {
            Ok(()) => {
                std::thread::sleep(Duration::from_millis(SETTLE_MS));
                return Ok(());
            }
            Err(err) => errors.push(err),
        }
    }
    Err(format!("no working clipboard helper ({})", errors.join("; ")))
}

const SETTLE_MS: u64 = 50;
const READ_TIMEOUT_MS: u64 = 200;
const WRITE_TIMEOUT_MS: u64 = 1000;
const MAX_READ_BYTES: usize = 16 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    // These are pipe-only helpers; tests never touch the user's clipboard.
    #[test]
    fn xclip_only_can_snapshot_text() {
        if std::env::var_os("XREMAP_TEST_XCLIP_ONLY").is_some() {
            assert_eq!(read().as_deref(), Some("original\n"));
            write("replacement").unwrap();
            return;
        }
        // Isolate PATH/display changes from the parallel test runner. Only
        // the fake xclip below can be invoked; no real clipboard is accessed.
        let dir = std::env::temp_dir().join(format!("xremap-xclip-{}-{:x}", std::process::id(), fastrand::u64(..)));
        std::fs::create_dir(&dir).unwrap();
        let helper = dir.join("xclip");
        std::fs::write(
            &helper,
            "#!/bin/sh\ncase \"$*\" in\n  '-selection clipboard -o') printf 'original\\n';;\n  '-selection clipboard') while IFS= read -r line; do :; done;;\n  *) exit 1;;\nesac\nexit 0\n",
        ).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "clipboard::tests::xclip_only_can_snapshot_text",
                "--nocapture",
            ])
            .env("XREMAP_TEST_XCLIP_ONLY", "1")
            .env("PATH", &dir)
            .env_remove("WAYLAND_DISPLAY");
        let x11 = command.output().unwrap();
        let wayland = command.env("WAYLAND_DISPLAY", "fake-review-display").output().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        for output in [x11, wayland] {
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn drains_output_larger_than_a_pipe_buffer() {
        let output =
            run_read("sh", &["-c", "exec dd if=/dev/zero bs=1024 count=256 2>/dev/null"], Duration::from_secs(2))
                .unwrap();
        assert_eq!(output.len(), 256 * 1024);
    }

    #[test]
    fn inherited_stdout_cannot_outlive_the_deadline() {
        let start = Instant::now();
        assert!(run_read("sh", &["-c", "sleep 2 &"], Duration::from_millis(80)).is_none());
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn stalled_stdin_is_bounded_too() {
        let input = vec![b'a'; 512 * 1024];
        let start = Instant::now();
        let error = run_helper("sh", &["-c", "exec sleep 2"], Some(&input), Duration::from_millis(80)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn child_failure_is_not_success() {
        assert!(run_read("sh", &["-c", "exit 1"], Duration::from_secs(1)).is_none());
    }
}
