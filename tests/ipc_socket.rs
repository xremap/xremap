#![cfg(feature = "device-test")]
//! Process-level IPC regressions require uinput permissions. No test emits
//! keys.

use std::io::{BufRead, BufReader, Write};
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
    fn start(inject: bool) -> Self {
        Self::start_with_config(inject, "")
    }

    /// `inject` gates the key commands; the mode tests do not need it.
    fn start_with_config(inject: bool, extra: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xremap-ipc-{}-{:x}", std::process::id(), fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.yml");
        let socket = dir.join("test.sock");
        let log = dir.join("daemon.log");
        std::fs::write(&config, format!("{extra}ipc:\n  socket: {}\n  inject: {}\n", socket.display(), inject))
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
