#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;
use crate::common::{assert_events, containsn, key_press, key_release};
use evdev::KeyCode;
use indoc::indoc;
use std::thread;
use std::time::Duration;

mod common;

#[test]
pub fn e2e_events_around_exit_action() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              keymap:
                - remap:
                    f12: { action: exit }
            "})?
        .build()?;

    // This is seen as one batch by xremap.
    ctrl.emit_events(&vec![
        // Keys before exit-action are handled
        key_press(KeyCode::KEY_A),
        key_press(KeyCode::KEY_F12),
        key_release(KeyCode::KEY_F12),
        // Keys after exit-action are dropped.
        key_press(KeyCode::KEY_K),
        key_release(KeyCode::KEY_K),
    ])?;

    // Note: KEY_A is held down at exit. But a release event is inserted
    //       by the kernel. This means clean up is very easy.
    assert_events(
        ctrl.fetch_until_end()?,
        indoc! {"
            a:1
            a:0
        "},
    );

    ctrl.wait_for_output()?;

    Ok(())
}

#[test]
pub fn e2e_actions_before_exit_action() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              keymap:
                - remap:
                    f12:
                        - A
                        - { action: exit }
            "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    assert_events(
        ctrl.fetch_until_end()?,
        indoc! {"
            a:1
            a:0
        "},
    );

    ctrl.wait_for_output()?;

    Ok(())
}

#[test]
pub fn e2e_full_reload_recreates_output_device() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              keymap:
                - remap:
                    f12: { action: reload }
            "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    ctrl.fetch_until_end()?; // Will timeout if device isn't closed.

    ctrl.kill()
}

#[test]
pub fn e2e_full_reload_cancels_at_config_error() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .allow_stdio_errors(true)
        .config(indoc! {"
              keymap:
                - remap:
                    f11: KEY_A
                    f12: { action: reload }
            "})?
        .build()?;

    std::fs::write(&ctrl.get_config_file(), "partial_config")?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    // Old config is still in effect.
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F11), key_release(KeyCode::KEY_F11)])?;

    assert_events(
        ctrl.fetch_until_key(KeyCode::KEY_A)?,
        indoc! {"
            a:1
            a:0
        "},
    );

    let stdout = ctrl.kill_for_output()?.stdout;
    assert!(containsn(1, &stdout, "Config error"));

    Ok(())
}

#[test]
pub fn e2e_full_reload_cancels_timers() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              experimental_map:
                - remap:
                    f11: { double: A }
              keymap:
                - remap:
                    f12: { action: reload }
            "})?
        .build()?;

    // These keys are buffered, and will be forgot because of reload.
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F11), key_release(KeyCode::KEY_F11)])?;

    // Reload
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12), key_release(KeyCode::KEY_F12)])?;
    ctrl.fetch_until_end()?; // Will timeout if device isn't closed.

    // Reopen device
    ctrl.forget_output_device()?;
    ctrl.open_output_device()?;

    // DoubleTapOperator will buffer KEY_MOVE sent by `fetch()`, so it's guaranteed
    //  that it has made a decision when KEY_MOVE is received here.
    assert_events(ctrl.fetch()?, "");

    ctrl.kill()
}

#[test]
pub fn e2e_mode_is_preserved_when_reloading_config() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              keymap:
                - remap:
                    f12:
                      - { set_mode: new_mode}
                      - { action: reload_config }
                - mode: new_mode
                  remap:
                    A: B
            "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12), key_release(KeyCode::KEY_F12)])?;

    thread::sleep(Duration::from_millis(50));

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_A), key_release(KeyCode::KEY_A)])?;

    assert_events(
        ctrl.fetch_until_key(KeyCode::KEY_B)?,
        indoc! {"
            b:1
            b:0
        "},
    );

    ctrl.kill()
}

#[test]
pub fn e2e_reload_config_inconsistency_example() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
              keymap:
                - remap:
                    f12: { action: reload_config }
            "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_A)])?;

    thread::sleep(Duration::from_millis(50));

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12), key_release(KeyCode::KEY_A)])?;

    // KEY_A is not released, because reload_config by-design squash all actions
    // that comes after it. It could be improved, but the problem of reloading
    // config consistently can't really be fixed with reasonable effort.
    assert_events(
        ctrl.fetch()?,
        indoc! {"
            a:1
        "},
    );

    ctrl.kill()
}
