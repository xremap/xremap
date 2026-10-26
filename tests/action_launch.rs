#![cfg(feature = "device-test")]

use crate::common::key_press;
use crate::common::xremap_controller::XremapController;
use evdev::KeyCode;
use indoc::indoc;
use std::thread::sleep;
use std::time::Duration;

mod common;

#[test]
pub fn e2e_launch_action() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
            keymap:
              - remap:
                  capslock: { launch: [echo, foo, bar] }
        "})?
        .env_var("DO_REAL_LAUNCH", "true")
        .build()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_CAPSLOCK)])?;

    sleep(Duration::from_millis(100));

    let output = ctrl.kill_for_output()?;

    assert!(output.stderr.contains(r#"Running command: ["echo", "foo", "bar"]"#));
    assert!(output.stderr.contains(r#"Process started: ["echo", "foo", "bar"]"#));

    Ok(())
}
