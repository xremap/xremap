#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use crate::common::{assert_err, assert_str_contains, get_random_device_name, get_raw_device_pair, wait_for_device};
use anyhow::Result;
use evdev::uinput::VirtualDevice;
use std::time::Duration;
use xremap::device::select_input_devices;

mod common;

#[test]
pub fn test_no_input_device_match() {
    let device_filter = vec!["match_nothing".into()];

    assert_err(
        "Failed to prepare input devices: No device was selected!",
        select_input_devices(&device_filter, &vec![], false, false, "own_device"),
    );
}

#[test]
pub fn test_device_without_keys_is_not_selected_automatically() -> Result<()> {
    // Create device without any output events.
    let name = get_random_device_name();
    let _device = VirtualDevice::builder()?.name(&name).build()?;
    let _ = wait_for_device(&name)?;

    // Automatically select devices
    let devices = select_input_devices(&[], &vec![], false, false, "own_device")?;

    assert_eq!(
        0,
        devices
            .iter()
            .filter(|(_, device)| device.device_name() == name)
            .count()
    );

    Ok(())
}

#[test]
pub fn test_device_filter_overwrites_keyboard_and_mouse_check() -> Result<()> {
    // Create device, that will not be selected automatically.
    let name = get_random_device_name();
    let _device = VirtualDevice::builder()?.name(&name).build()?;
    let _ = wait_for_device(&name)?;

    // Selects the device, because filter overwrites the automatic selection rules.
    let names: Vec<String> = select_input_devices(&[name.clone()], &vec![], false, false, "own_device")?
        .iter()
        .map(|(_, device)| device.device_name().to_string())
        .collect();

    assert_eq!(vec![name], names);

    Ok(())
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
