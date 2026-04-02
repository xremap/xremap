#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;
use crate::common::{assert_events, final_event_state, key_press, key_release};
use anyhow::Result;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::thread;
use std::time::Duration;

mod common;

#[test]
pub fn e2e_modmap_no_op() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_Q)])?;

    assert_eq!(Some(1), final_event_state(Key::KEY_Q, ctrl.fetch_events()?));

    ctrl.kill()
}

#[test]
fn e2e_modmap_multipurpose_key_alone() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                modmap:
                    - remap:
                        CapsLock:
                          held: CapsLock
                          alone: Esc
                "})?
        .build()?;

    for _ in 0..10 {
        if fastrand::bool() {
            ctrl.emit_events(&vec![key_press(Key::KEY_CAPSLOCK), key_release(Key::KEY_CAPSLOCK)])?;
        } else {
            ctrl.emit_events(&vec![key_press(Key::KEY_CAPSLOCK)])?;

            thread::sleep(Duration::from_millis(fastrand::u64(0..5)));

            ctrl.emit_events(&vec![key_release(Key::KEY_CAPSLOCK)])?;
        }

        assert_events(
            ctrl.fetch()?,
            indoc! {"
                esc:1
                esc:0
            "},
        );
    }

    ctrl.kill()
}
