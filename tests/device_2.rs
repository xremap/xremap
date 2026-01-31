#![cfg(feature = "device-test")]

use crate::common::{get_random_device_name, wait_for_device};
use anyhow::Result;
use evdev::uinput::VirtualDevice;
use xremap::device::get_input_devices;

mod common;

// Must run alone, because `get_input_devices` mutates global variable.
#[test]
pub fn test_device_without_keys_is_not_selected_automatically() -> Result<()> {
    // Create device without any output events.
    let name = get_random_device_name();
    let _device = VirtualDevice::builder()?.name(&name).build()?;
    let _ = wait_for_device(&name)?;

    // Automatically select devices
    let devices = get_input_devices(&[], &vec![], false, false)?;

    assert_eq!(
        0,
        devices
            .iter()
            .filter(|(_, device)| device.device_name() == name)
            .count()
    );

    Ok(())
}
