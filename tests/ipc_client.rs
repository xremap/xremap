//! Client framing and validation tests do not require input devices.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn assert_rejected_before_connect(args: &[&str]) {
    let socket =
        std::env::temp_dir().join(format!("xremap-missing-{}-{:x}.sock", std::process::id(), fastrand::u64(..)));
    let output = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .arg("--socket")
        .arg(socket)
        .args(args)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IPC command must be a single line"), "{stderr}");
    assert!(!stderr.contains("cannot connect"), "{stderr}");
}

#[test]
fn mode_arguments_cannot_inject_protocol_lines() {
    for mode in [
        "normal\nset-mode injected",
        "normal\rset-mode injected",
        "normal\r\nset-mode injected",
    ] {
        assert_rejected_before_connect(&["set-mode", mode]);
    }
}

#[test]
fn type_arguments_cannot_inject_protocol_lines() {
    for text in [
        "hello\nset-mode injected",
        "hello\rset-mode injected",
        "hello\r\nset-mode injected",
    ] {
        assert_rejected_before_connect(&["type", text]);
    }
    for option in ["--layout", "--variant"] {
        assert_rejected_before_connect(&["type", option, "us\nset-mode injected", "--", "text"]);
    }
    assert_rejected_before_connect(&["type", "--base64", "YQ==\nset-mode injected"]);
}

#[test]
fn click_arguments_cannot_inject_protocol_lines() {
    assert_rejected_before_connect(&["click", "left\nset-mode injected"]);
}

#[test]
fn key_arguments_cannot_inject_protocol_lines() {
    for verb in ["key", "keydown", "keyup"] {
        assert_rejected_before_connect(&[verb, "raw", "f13\nset-mode injected"]);
        assert_rejected_before_connect(&[verb, "--device", "kbd\nset-mode injected", "raw", "f13"]);
    }
}

fn assert_sent_line(args: &[&str], input: Option<&[u8]>, expected: &str) {
    let dir = std::env::temp_dir().join(format!("xremap-client-{}-{:x}", std::process::id(), fastrand::u64(..)));
    std::fs::create_dir(&dir).unwrap();
    let socket = dir.join("control.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(err) => panic!("mock socket accept failed: {err}"),
            }
        };
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut line = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut line)
            .unwrap();
        stream.write_all(b"ok\n").unwrap();
        line
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_xremap"))
        .arg("--socket")
        .arg(&socket)
        .args(args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child.stdin.take().unwrap().write_all(input).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    let received = server.join().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok\n");
    assert_eq!(received, expected);
}

#[test]
fn type_stdin_encodes_normalized_newlines_as_one_command() {
    use base64::Engine;
    let payload = base64::engine::general_purpose::STANDARD.encode("a\nb\nc\nd");
    assert_sent_line(&["type", "--stdin"], Some(b"a\r\nb\rc\nd"), &format!("type --base64 {payload}\n"));
}

#[test]
fn type_escaped_newlines_and_literal_options_stay_on_one_line() {
    assert_sent_line(&["type", "--", r"a\nb"], None, "type -- a\\nb\n");
    assert_sent_line(&["type", "--", "--socket", "literal"], None, "type -- --socket literal\n");
}
