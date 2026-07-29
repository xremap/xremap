#![cfg(feature = "device-test")]

use crate::common::{get_random_device_name, wait_for_device};
use anyhow::Result;
use evdev::uinput::VirtualDevice;
use xremap::private::select_input_devices;

mod common;

// Keep this test in its own integration-test binary because it grabs all
// keyboards (see https://github.com/xremap/xremap/pull/967).
#[test]
pub fn test_device_without_keys_is_not_selected_automatically() -> Result<()> {
    // Create a device without any output events.
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
