#![cfg(feature = "device-test")]

use crate::common::{assert_events, key_press, xremap_controller::XremapController};
use evdev::KeyCode;
use indoc::indoc;

mod common;

#[test]
pub fn e2e_keymap_no_op() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_W)])?;

    assert_events(
        ctrl.fetch_until_key(KeyCode::KEY_W)?,
        indoc! {"
            w:1
            w:0
        "},
    );

    ctrl.kill()
}
