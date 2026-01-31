#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use crate::common::{assert_err, assert_str_contains, get_raw_device_pair};
use anyhow::Result;
use std::time::Duration;
use xremap::device::get_input_devices;

mod common;

#[test]
pub fn test_no_input_device_match() {
    let device_filter = vec!["match_nothing".into()];

    assert_err("No device was selected!", get_input_devices(&device_filter, &vec![], false, false));
}

#[test]
pub fn test_device_that_does_not_exist() -> Result<()> {
    // The device path doesn't exist, so will not cause other errors than no devices selected
    let ctrl = XremapController::builder()
        .not_open_for_fetch()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "/dev/input/event99".into(),
        })
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert_str_contains("Failed to prepare input devices", &output.stderr);

    Ok(())
}

#[test]
pub fn test_device_that_is_already_grabbed() -> Result<()> {
    let (mut input, output) = get_raw_device_pair()?;

    let mut ctrl = XremapController::builder()
        .not_open_for_fetch()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: output.path.to_string_lossy().into(),
        })
        .build()?;

    std::thread::sleep(Duration::from_millis(1000));

    input.ungrab()?;

    let output = ctrl.kill_for_output()?;

    assert_str_contains("Failed to prepare input devices", &output.stderr);
    assert_str_contains("Device or resource busy", &output.stderr);

    Ok(())
}
