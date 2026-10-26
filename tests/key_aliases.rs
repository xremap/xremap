#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;
use crate::common::{assert_events, key_press};
use anyhow::Result;
use evdev::KeyCode as Key;
use indoc::indoc;

mod common;

// key_aliases are implemented with a static mut, which
// makes units harder to write. These e2e tests have
// their own process, so they work fine.

#[test]
fn e2e_key_aliases() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                key_aliases:
                    λ: KEY_L
                keymap:
                    - remap:
                        A: λ
                        λ: A
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_A)])?;
    ctrl.emit_events(&vec![key_press(Key::KEY_L)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
                l:1
                l:0
                a:1
                a:0
        "},
    );

    ctrl.kill()
}

#[test]
fn e2e_key_aliases_are_case_insensitive() -> Result<()> {
    let mut ctrl = XremapController::builder()
        .config(indoc! {"
                key_aliases:
                    omega: O
                    Ω: K        # not used
                    ω: O        # overwrites uppercase omega
                keymap:
                    - remap:
                        A: omega
                        B: ω
                        C: Ω
                "})?
        .build()?;

    ctrl.emit_events(&vec![key_press(Key::KEY_A)])?;
    ctrl.emit_events(&vec![key_press(Key::KEY_B)])?;
    ctrl.emit_events(&vec![key_press(Key::KEY_C)])?;

    assert_events(
        ctrl.fetch()?,
        indoc! {"
            	o:1
            	o:0
            	o:1
            	o:0
            	o:1
            	o:0
        "},
    );

    ctrl.kill()
}
