#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use indoc::indoc;

mod common;

#[test]
pub fn e2e_device_filter_does_not_match() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
        .allow_stdio_errors(true)
        .not_open_for_fetch()
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert!(output
        .stderr
        .contains("Error: Failed to prepare input devices: No device was selected!"));

    ctrl.kill()
}

#[test]
pub fn e2e_validate_config_does_not_start_xremap() -> anyhow::Result<()> {
    // The device filter would make xremap fail, if it did start.
    let ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
        .not_open_for_fetch()
        .validate_config(true)
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert_eq!(output.stdout, "Config is valid\n");

    ctrl.kill()
}

#[test]
pub fn e2e_validate_config_fails_on_invalid_config() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
        .allow_stdio_errors(true)
        .not_open_for_fetch()
        .validate_config(true)
        .config(indoc! {"
              keymap:
                - remap:
                    f12: not_a_key
            "})?
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert_eq!(output.stdout, "");
    assert!(output.stderr.contains("Error: Failed to load config"));

    ctrl.kill()
}
