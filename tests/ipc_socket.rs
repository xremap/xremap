#![cfg(feature = "device-test")]
//! Process-level IPC regressions require uinput permissions. No test emits
//! keys or accesses the real clipboard; failing clipboard helpers are local.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn connect(socket: &Path) -> std::io::Result<UnixStream> {
    let stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    Ok(stream)
}

fn exchange(stream: &mut UnixStream, commands: &str, count: usize) -> Vec<String> {
    stream.write_all(commands.as_bytes()).unwrap();
    let mut reader = BufReader::new(stream);
    (0..count)
        .map(|_| {
            let mut reply = String::new();
            reader.read_line(&mut reply).unwrap();
            assert!(!reply.is_empty(), "connection closed before replying");
            reply
        })
        .collect()
}

fn send(socket: &Path, command: &str) -> String {
    exchange(&mut connect(socket).unwrap(), &format!("{command}\n"), 1)
        .pop()
        .unwrap()
}

struct DaemonGuard {
    child: Child,
    dir: PathBuf,
    socket: PathBuf,
    log: PathBuf,
}

impl DaemonGuard {
    fn start(stub_clipboard: bool) -> Self {
        Self::start_with_config(stub_clipboard, "")
    }

    /// `stub_clipboard` enables injection; the mode tests do not need it.
    fn start_with_config(stub_clipboard: bool, extra: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xremap-ipc-{}-{:x}", std::process::id(), fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.yml");
        let socket = dir.join("test.sock");
        let log = dir.join("daemon.log");
        std::fs::write(&config, format!("{extra}ipc:\n  socket: {}\n  inject: {}\n", socket.display(), stub_clipboard))
            .unwrap();
        let logfile = std::fs::File::create(&log).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_xremap"));
        command
            .arg(&config)
            .args([
                "--device",
                "nonexistent-ipc-test-device",
                "--watch=device",
                "--desktop",
                "none",
            ])
            .stdout(Stdio::from(logfile.try_clone().unwrap()))
            .stderr(Stdio::from(logfile));
        if stub_clipboard {
            let helpers = dir.join("helpers");
            std::fs::create_dir(&helpers).unwrap();
            for name in ["wl-copy", "xsel", "xclip"] {
                let helper = helpers.join(name);
                std::fs::write(&helper, "#!/bin/sh\nexit 1\n").unwrap();
                std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            command.env("PATH", helpers);
        }
        let child = command.spawn().unwrap();
        let mut guard = Self {
            child,
            dir,
            socket: socket.clone(),
            log,
        };
        guard.wait_for_socket(&socket);
        guard
    }

    fn wait_for_socket(&mut self, socket: &Path) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("daemon exited with {status}: {}", std::fs::read_to_string(&self.log).unwrap());
            }
            if let Ok(mut stream) = connect(socket) {
                assert_eq!(exchange(&mut stream, "ping\n", 1), ["pong\n"]);
                return;
            }
            assert!(
                Instant::now() < deadline,
                "daemon/socket not ready: {}",
                std::fs::read_to_string(&self.log).unwrap()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn mode_replies_escape_config_values_without_desynchronizing_clients() {
    let daemon = DaemonGuard::start_with_config(false, "default_mode: \"first\\nsecond\\rthird\\\\four\"\n");
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "get-mode\nping\nget-mode\n", 3);
    assert_eq!(
        replies,
        [
            "first\\nsecond\\rthird\\\\four\n",
            "pong\n",
            "first\\nsecond\\rthird\\\\four\n"
        ]
    );
}

#[test]
fn partial_reload_preserves_pipelined_commands() {
    let daemon = DaemonGuard::start_with_config(true, "keymap:\n  - remap:\n      f13: { action: reload_config }\n");
    let mut stream = connect(&daemon.socket).unwrap();
    assert_eq!(
        exchange(&mut stream, "keydown keymap f13\nset-mode after-reload\nget-mode\nping\n", 4),
        ["ok\n", "ok\n", "after-reload\n", "pong\n"]
    );
    assert_eq!(exchange(&mut stream, "ping\n", 1), ["pong\n"]);
}

#[test]
fn pending_commands_observe_the_reloaded_injection_gate() {
    let daemon = DaemonGuard::start_with_config(true, "keymap:\n  - remap:\n      f13: { action: reload_config }\n");
    std::fs::write(
        daemon.dir.join("config.yml"),
        format!("ipc:\n  socket: {}\n  inject: false\n", daemon.socket.display()),
    )
    .unwrap();
    let mut stream = connect(&daemon.socket).unwrap();
    assert_eq!(
        exchange(&mut stream, "keydown keymap f13\nkeydown keymap f13\nping\n", 3),
        ["ok\n", "err injection disabled\n", "pong\n"]
    );
}

#[test]
fn failed_partial_reload_keeps_pipelined_commands_usable() {
    let daemon = DaemonGuard::start_with_config(true, "keymap:\n  - remap:\n      f13: { action: reload_config }\n");
    std::fs::write(daemon.dir.join("config.yml"), "keymap: [\n").unwrap();
    let mut stream = connect(&daemon.socket).unwrap();
    assert_eq!(
        exchange(&mut stream, "keydown keymap f13\nset-mode kept\nget-mode\nping\n", 4),
        ["ok\n", "ok\n", "kept\n", "pong\n"]
    );
}

#[test]
fn failed_listener_replacement_keeps_pipelined_commands_usable() {
    let daemon = DaemonGuard::start_with_config(true, "keymap:\n  - remap:\n      f13: { action: reload_config }\n");
    let occupied = daemon.dir.join("ordinary-file");
    std::fs::write(&occupied, "keep this file").unwrap();
    std::fs::write(daemon.dir.join("config.yml"), format!("ipc:\n  socket: {}\n", occupied.display())).unwrap();
    let mut stream = connect(&daemon.socket).unwrap();
    assert_eq!(
        exchange(&mut stream, "keydown keymap f13\nset-mode kept\nget-mode\nping\n", 4),
        ["ok\n", "ok\n", "kept\n", "pong\n"]
    );
    assert_eq!(std::fs::read_to_string(occupied).unwrap(), "keep this file");
}

#[test]
fn pending_commands_survive_listener_replacement_and_disabling_ipc() {
    for enabled in [true, false] {
        let daemon =
            DaemonGuard::start_with_config(true, "keymap:\n  - remap:\n      f13: { action: reload_config }\n");
        let replacement = daemon.dir.join("replacement.sock");
        let config = if enabled {
            format!("ipc:\n  socket: {}\n", replacement.display())
        } else {
            "ipc:\n  socket: false\n".to_string()
        };
        std::fs::write(daemon.dir.join("config.yml"), config).unwrap();
        let mut stream = connect(&daemon.socket).unwrap();
        let expected = if enabled { "pong\n" } else { "err IPC disabled\n" };
        assert_eq!(exchange(&mut stream, "keydown keymap f13\nping\n", 2), ["ok\n", expected]);
        assert!(!daemon.socket.exists());
        if enabled {
            assert_eq!(send(&replacement, "ping"), "pong\n");
        }
    }
}

#[test]
fn subcommand_sends_one_command_and_prints_the_reply() {
    let daemon = DaemonGuard::start(false);
    let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .arg("--socket")
        .arg(&daemon.socket)
        .arg("set-mode")
        .arg("normal")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok\n");
    assert_eq!(send(&daemon.socket, "get-mode"), "normal\n");
}

#[test]
fn pointer_subcommands_parse_negative_values_and_use_the_trailing_socket() {
    // The disabled injection gate checks client parsing without moving the pointer.
    let daemon = DaemonGuard::start(false);
    for args in [
        vec!["wheel", "-2"],
        vec!["hwheel", "-1"],
        vec!["mousemove", "4", "-4"],
        vec!["mousemove", "-4", "4"],
        vec!["click", "left", "right"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
            .args(args)
            .arg("--socket")
            .arg(&daemon.socket)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, b"err injection disabled\n", "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn mouse_and_wheel_commands_are_dispatched_without_touching_keys() {
    let mut daemon = DaemonGuard::start(true);
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "wheel -2\nhwheel 1\nmousemove 4 -4\nclick left middle right\nping\n", 5);
    assert_eq!(replies, ["ok\n", "ok\n", "ok\n", "ok\n", "pong\n"]);
    assert!(daemon.child.try_wait().unwrap().is_none());

    // Parse errors are command-local.
    let replies = exchange(&mut stream, "click up\nwheel x\nmousemove 1\nping\n", 4);
    assert_eq!(
        replies,
        [
            "err unknown button: up (left/middle/right)\n",
            "err wheel takes an integer amount\n",
            "err mousemove takes X and Y coordinates\n",
            "pong\n"
        ]
    );
}

#[test]
fn wheel_fails_loudly_when_enable_wheel_is_false() {
    // The output device then has no REL_WHEEL/HWHEEL axis, and an emit on an
    // unregistered axis is silently dropped, so the command must not reply `ok`.
    let mut daemon = DaemonGuard::start_with_config(true, "enable_wheel: false\n");
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "wheel 1\nhwheel 1\nmousemove 1 1\nclick left\nping\n", 5);
    let wheel_error = "err wheel events are disabled (set `enable_wheel: true` to inject them)\n";
    assert_eq!(replies, [wheel_error, wheel_error, "ok\n", "ok\n", "pong\n"],);
    assert!(daemon.child.try_wait().unwrap().is_none());
}

#[test]
fn type_subcommand_uses_the_socket_option_after_text() {
    let daemon = DaemonGuard::start(false);
    let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .args(["type", "hello", "world", "--socket"])
        .arg(&daemon.socket)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"err injection disabled\n", "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn type_stdin_with_inline_text_is_rejected_locally() {
    // A conflicting invocation must fail in the CLI, before anything reaches
    // the server, so no text can be typed into the session.
    let daemon = DaemonGuard::start(false);
    let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .arg("--socket")
        .arg(&daemon.socket)
        .args(["type", "--stdin", "extra"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "must not reach the server");
}

#[test]
fn invalid_base64_is_reported_by_the_server() {
    let mut daemon = DaemonGuard::start(true);
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "type --base64 !!!\ntype --base64 //4=\nping\n", 3);
    assert!(replies[0].starts_with("err invalid base64:"), "{replies:?}");
    assert_eq!(replies[1], "err base64 payload is not valid UTF-8\n");
    assert_eq!(replies[2], "pong\n");
    assert!(daemon.child.try_wait().unwrap().is_none());
}

#[test]
fn clipboard_failure_is_reported_without_exiting_daemon() {
    let mut daemon = DaemonGuard::start(true);
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "type --method clipboard 中文\nping\n", 2);
    assert!(replies[0].starts_with("err failed to write clipboard: no working clipboard helper"), "{replies:?}");
    assert_eq!(replies[1], "pong\n");
    assert!(daemon.child.try_wait().unwrap().is_none());
}
