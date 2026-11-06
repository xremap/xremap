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
    config: PathBuf,
    socket: PathBuf,
    log: PathBuf,
}

impl DaemonGuard {
    fn start(inject: bool) -> Self {
        Self::start_with_config(inject, "", false)
    }

    /// `inject` gates the key commands; the reload and mode tests do not need it.
    fn start_with_config(inject: bool, extra: &str, watch_config: bool) -> Self {
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
                if watch_config {
                    "--watch=config,device"
                } else {
                    "--watch=device"
                },
                "--desktop",
                "none",
            ])
            .stdout(Stdio::from(logfile.try_clone().unwrap()))
            .stderr(Stdio::from(logfile));
        let child = command.spawn().unwrap();
        let mut guard = Self {
            child,
            dir,
            config,
            socket: socket.clone(),
            log,
        };
        guard.wait_for_socket(&socket);
        guard
    }

    fn wait_for_log(&mut self, message: &str, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(self.child.try_wait().unwrap().is_none(), "daemon exited");
            let log = std::fs::read_to_string(&self.log).unwrap();
            if log.matches(message).count() >= count {
                return;
            }
            assert!(Instant::now() < deadline, "did not see {message:?}: {log}");
            std::thread::sleep(Duration::from_millis(20));
        }
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
fn socket_survives_a_config_reload() {
    let mut daemon = DaemonGuard::start(false);
    let mut stream = connect(&daemon.socket).unwrap();
    // One write forces set/get/reload/get to share the same input batch.
    assert_eq!(
        exchange(&mut stream, "set-mode after_reload\nget-mode\nreload-config\nget-mode\nping\n", 5),
        ["ok\n", "after_reload\n", "ok\n", "after_reload\n", "pong\n"]
    );

    // A failed reload must be an error reply, retaining the old configuration,
    // mode, listener, and the remaining commands in this same batch.
    std::fs::write(&daemon.config, "ipc: [\n").unwrap();
    let replies = exchange(&mut stream, "reload-config\nget-mode\nping\n", 3);
    assert!(replies[0].starts_with("err Failed to load config:"), "{replies:?}");
    assert_eq!(&replies[1..], ["after_reload\n", "pong\n"]);

    // A bind failure must also keep the current socket usable.
    let invalid_path = daemon.dir.join("missing/test.sock");
    std::fs::write(&daemon.config, format!("ipc:\n  socket: {}\n", invalid_path.display())).unwrap();
    assert!(send(&daemon.socket, "reload-config").starts_with("err Failed to start IPC server"));
    assert_eq!(send(&daemon.socket, "ping"), "pong\n");

    // Successful path changes reply over the old connection, then accept new
    // clients at the new path and unlink the previous socket file.
    let socket2 = daemon.dir.join("test2.sock");
    std::fs::write(&daemon.config, format!("ipc:\n  socket: {}\n", socket2.display())).unwrap();
    assert_eq!(send(&daemon.socket, "reload-config"), "ok\n");
    daemon.wait_for_socket(&socket2);
    assert!(!daemon.socket.exists());
    assert_eq!(send(&socket2, "get-mode"), "after_reload\n");
}

#[test]
fn failed_reloads_preserve_key_aliases() {
    let daemon = DaemonGuard::start_with_config(true, "key_aliases:\n  oldalias: f13\n", false);
    let mut stream = connect(&daemon.socket).unwrap();
    // Too many chords is a parse-only probe: alias lookup succeeds, but no
    // key events are executed. An unknown alias instead fails key parsing.
    let probe = "key oldalias*16 f14\n";
    assert_eq!(exchange(&mut stream, probe, 1), ["err too many chords (max 16)\n"]);
    let invalid_path = daemon.dir.join("missing/control.sock");
    let failures = [
        ("ipc: [\n".to_string(), "err Failed to load config:"),
        ("key_aliases:\n  newalias: f14\nunknown_field: true\n".to_string(), "err Failed to load config:"),
        (
            format!("key_aliases:\n  newalias: f14\nipc:\n  socket: {}\n  inject: true\n", invalid_path.display()),
            "err Failed to start IPC server",
        ),
    ];
    for (config, error) in failures {
        std::fs::write(&daemon.config, config).unwrap();
        let replies = exchange(&mut stream, &format!("reload-config\n{probe}ping\n"), 3);
        assert!(replies[0].starts_with(error), "{replies:?}");
        assert_eq!(&replies[1..], ["err too many chords (max 16)\n", "pong\n"]);
        assert!(send(&daemon.socket, "key newalias*16 f14").starts_with("err invalid key:"));
    }

    // Successful reloads must commit the new aliases rather than roll back.
    std::fs::write(
        &daemon.config,
        format!("key_aliases:\n  newalias: f14\nipc:\n  socket: {}\n  inject: true\n", daemon.socket.display()),
    )
    .unwrap();
    assert_eq!(send(&daemon.socket, "reload-config"), "ok\n");
    assert_eq!(send(&daemon.socket, "key newalias*16 f14"), "err too many chords (max 16)\n");
    assert!(send(&daemon.socket, "key oldalias*16 f14").starts_with("err invalid key:"));
}

#[cfg(target_os = "linux")]
#[test]
fn watcher_failures_preserve_key_aliases() {
    let mut daemon = DaemonGuard::start_with_config(true, "key_aliases:\n  oldalias: f13\n", true);
    let invalid_path = daemon.dir.join("missing/control.sock");
    let failures = [
        ("ipc: [\n".to_string(), "Config error: Failed to load config:"),
        (
            format!("key_aliases:\n  newalias: f14\nipc:\n  socket: {}\n  inject: true\n", invalid_path.display()),
            "Config error: Failed to start IPC server",
        ),
    ];
    for (config, error) in failures {
        std::fs::write(&daemon.config, config).unwrap();
        daemon.wait_for_log(error, 1);
        assert_eq!(send(&daemon.socket, "key oldalias*16 f14"), "err too many chords (max 16)\n");
        assert!(send(&daemon.socket, "key newalias*16 f14").starts_with("err invalid key:"));
    }
}

#[test]
fn mode_replies_escape_config_values_without_desynchronizing_clients() {
    let daemon = DaemonGuard::start_with_config(false, "default_mode: \"first\\nsecond\\rthird\\\\four\"\n", false);
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
fn client_accepts_a_complete_command_as_one_argument() {
    let daemon = DaemonGuard::start(false);
    let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .arg("--socket")
        .arg(&daemon.socket)
        .arg("--client")
        .arg("set-mode normal")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok\n");
    assert_eq!(send(&daemon.socket, "get-mode"), "normal\n");
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
    let mut daemon = DaemonGuard::start_with_config(true, "enable_wheel: false\n", false);
    let mut stream = connect(&daemon.socket).unwrap();
    let replies = exchange(&mut stream, "wheel 1\nhwheel 1\nmousemove 1 1\nclick left\nping\n", 5);
    let wheel_error = "err wheel events are disabled (set `enable_wheel: true` to inject them)\n";
    assert_eq!(
        replies,
        [wheel_error, wheel_error, "ok\n", "ok\n", "pong\n"],
    );
    assert!(daemon.child.try_wait().unwrap().is_none());
}
