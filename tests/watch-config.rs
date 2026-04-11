#![cfg(feature = "device-test")]

use crate::common::{assert_events, containsn, key_press, key_release, xremap_controller::XremapController};
use evdev::KeyCode;
use indoc::indoc;
mod common;

#[test]
pub fn e2e_watch_config_cur() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder().watch_config("")?.build()?;

    std::fs::write(
        &ctrl.get_config_file(),
        indoc! {"
          keymap:
            - remap:
                f12: key_1
        "},
    )?;

    std::thread::sleep(std::time::Duration::from_millis(20));

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            1:1
            1:0
        "},
    );

    assert!(ctrl.kill_for_output()?.stdout.contains("Reloading Config"));

    Ok(())
}

#[test]
pub fn e2e_old_config_remains_active_when_error_cur() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .watch_config(indoc! {"
              config_watch_debounce_ms: 10
              keymap:
                - remap:
                    f12: key_1
            "})?
        .build()?;

    std::fs::write(&ctrl.get_config_file(), "failed_config")?;

    std::thread::sleep(std::time::Duration::from_millis(20));

    // This is fragile without debounce, because the file write can cause
    // two events one with an empty file and one with the new content.
    // This means xremap drops the old and replace it with a 'blank' config,
    // instead of leaving the old in place.
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12), key_release(KeyCode::KEY_F12)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            1:1
            1:0
        "},
    );

    // Successful config can be restored
    std::fs::write(
        &ctrl.get_config_file(),
        indoc! {"
          keymap:
            - remap:
                f12: key_2
        "},
    )?;

    std::thread::sleep(std::time::Duration::from_millis(20));

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            2:1
            2:0
        "},
    );

    ctrl.kill()
}

#[test]
pub fn e2e_config_watch_is_debounced_cur() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .watch_config("config_watch_debounce_ms: 10")?
        .build()?;

    std::fs::write(&ctrl.get_config_file(), "")?;
    std::fs::write(&ctrl.get_config_file(), "failed_config")?;
    std::fs::write(&ctrl.get_config_file(), "")?;
    std::fs::write(&ctrl.get_config_file(), "other problem")?;
    std::fs::write(
        &ctrl.get_config_file(),
        indoc! {"
        keymap:
          - remap:
              f12: key_1
        "},
    )?;

    std::thread::sleep(std::time::Duration::from_millis(20));

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            1:1
            1:0
        "},
    );

    let stdout = ctrl.kill_for_output()?.stdout;

    assert!(containsn(1, &stdout, "Reloading Config"));

    Ok(())
}
