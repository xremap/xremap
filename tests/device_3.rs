#![cfg(feature = "device-test")]

use crate::common::{get_random_device_name, wait_for_device};
use anyhow::Result;
use evdev::uinput::VirtualDevice;
use xremap::device::get_input_devices;

mod common;

// Must run alone, because `get_input_devices` mutates global variable.
#[test]
pub fn test_device_filter_overwrites_keyboard_and_mouse_check() -> Result<()> {
    // Create device, that will not be selected automatically.
    let name = get_random_device_name();
    let _device = VirtualDevice::builder()?.name(&name).build()?;
    let _ = wait_for_device(&name)?;

    // Selects the device, because filter overwrites the automatic selection rules.
    let names: Vec<String> = get_input_devices(&[name.clone()], &vec![], false, false)?
        .iter()
        .map(|(_, device)| device.device_name().to_string())
        .collect();

    assert_eq!(vec![name], names);

    Ok(())
}
