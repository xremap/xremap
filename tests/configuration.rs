#![cfg(feature = "device-test")]

use crate::common::xremap_controller::XremapController;

mod common;

#[test]
pub fn e2e_device_filter_does_not_match() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .custom_input_device("match_nothing")
        .not_open_for_fetch()
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert!(output
        .stderr
        .contains("Error: Failed to prepare input devices: No device was selected!"));

    ctrl.kill()
}
