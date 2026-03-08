#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;
use crate::common::{assert_events, key_press, key_release};
use anyhow::Result;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::thread;
use std::time::Duration;

mod common;

#[test]
fn e2e_test_sim_operator_matches() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                experimental_map:
                    - chords:
                        - keys: [kp7, kp8]
                          actions: key_3
                          timeout: 1000
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_KP7)])?;
    ctrl.emit_events(&vec![key_press(Key::KEY_KP8)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP7)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP8)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            3:1
            3:0
        "},
    );

    ctrl.kill()
}

#[test]
fn e2e_test_sim_operator_cancels() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                experimental_map:
                    - chords:
                        - keys: [kp7, kp8]
                          actions: key_3
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_KP7)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP7)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            kp7:1
            kp7:0
        "},
    );

    ctrl.kill()
}

#[test]
fn e2e_test_sim_operator_times_out() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                experimental_map:
                    - chords:
                        - keys: [kp7, kp8]
                          actions: key_3
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_KP7)])?;

    // the timeout is flacky, because it's not synchronized to when
    // the operator receives the first key press.
    thread::sleep(Duration::from_millis(50));

    ctrl.emit_events(&vec![key_release(Key::KEY_KP7)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            kp7:1
            kp7:0
        "},
    );

    ctrl.kill()
}

#[test]
fn e2e_test_dbltap_operator_matches() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                experimental_map:
                    - remap:
                        kp9: 
                            double: key_6
                            timeout: 1000
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_KP9)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP9)])?;
    ctrl.emit_events(&vec![key_press(Key::KEY_KP9)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP9)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            6:1
            6:0
        "},
    );

    ctrl.kill()
}

#[test]
fn e2e_test_dbltap_operator_times_out() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                experimental_map:
                    - remap:
                        kp9: 
                            double: key_6
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_KP9)])?;
    ctrl.emit_events(&vec![key_release(Key::KEY_KP9)])?;

    // the timeout is flacky, because it's not synchronized to when
    // the operator receives the first key press.
    thread::sleep(Duration::from_millis(200));

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            kp9:1
            kp9:0
        "},
    );

    ctrl.kill()
}
