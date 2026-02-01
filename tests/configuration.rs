#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};

mod common;

#[test]
pub fn e2e_device_filter_does_not_match() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
        .not_open_for_fetch()
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert!(output
        .stderr
        .contains("Error: Failed to prepare input devices: No device was selected!"));

    ctrl.kill()
}
