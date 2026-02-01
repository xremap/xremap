#![cfg(feature = "device-test")]

use crate::common::{final_event_state, key_press, xremap_controller::XremapController};
use evdev::KeyCode;

mod common;

#[test]
pub fn e2e_modmap_no_op() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_Q)])?;

    assert_eq!(Some(1), final_event_state(KeyCode::KEY_Q, ctrl.fetch_events()?));

    ctrl.kill()
}

#[test]
pub fn e2e_keymap_no_op() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    ctrl.emit_events(&vec![key_press(KeyCode::KEY_W)])?;

    // keymap clicks when triggered so ends in released state.
    assert_eq!(Some(0), final_event_state(KeyCode::KEY_W, ctrl.fetch_events()?));

    ctrl.kill()
}
