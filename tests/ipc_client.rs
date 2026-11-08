//! Client validation must happen before connecting to a daemon.

use std::process::Command;

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
