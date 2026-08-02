#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;
use crate::common::{assert_events, key_press, key_release};
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
