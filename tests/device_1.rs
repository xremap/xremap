#![cfg(feature = "device-test")]

use crate::common::get_virtual_device;
use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use anyhow::Result;

mod common;

// Must run alone, because it doesn't have a device filter.
#[test]
pub fn test_output_device_from_other_xremap_process_is_grabbed() -> Result<()> {
    let devices_at_test_start = evdev::enumerate()
        .into_iter()
        .map(|(path, _)| path.display().to_string())
        .collect::<Vec<_>>()
        .join(",");

    let _dev = get_virtual_device("xremap")?; // Simulate other process running.

    let ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::NoFilter)
        .ignore_device(devices_at_test_start)
        .build()?;

    ctrl.raw_kill()?;

    Ok(())
}
