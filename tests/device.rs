#![cfg(feature = "device-test")]

use crate::common::{assert_err, get_random_device_name, get_virtual_device};
use anyhow::Result;
use xremap::device::get_input_devices;

mod common;

#[test]
pub fn test_no_input_device_match() {
    let device_filter = vec!["match_nothing".into()];

    assert_err("No device was selected!", get_input_devices(&device_filter, &vec![], false, false));
}

#[test]
pub fn test_device_filter_overwrites_keyboard_and_mouse_check() -> Result<()> {
    // Create device, that will not be selected automatically.
    let name = get_random_device_name();
    let _device = get_virtual_device(&name)?;

    // Selects the device, because filter overwrites the automatic selection rules.
    let names: Vec<String> = get_input_devices(&[name.clone()], &vec![], false, false)?
        .iter()
        .map(|(_, device)| device.device_name().to_string())
        .collect();

    assert_eq!(vec![name], names);

    Ok(())
}
