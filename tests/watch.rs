#![cfg(feature = "device-test")]

use crate::common::{get_random_device_name, get_virtual_device, xremap_controller::XremapController};

mod common;

#[test]
pub fn e2e_match_nothing_in_watch_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .custom_input_device("match_nothing")
        .watch(true)
        .build()?;

    assert!(ctrl
        .kill_for_output()?
        .stdout
        .contains("No device was selected, but --watch is waiting for new devices."));

    ctrl.kill()
}

#[test]
pub fn e2e_connecting_device_in_watch_mode() -> anyhow::Result<()> {
    let name = get_random_device_name();

    let mut ctrl = XremapController::builder()
        .custom_input_device(&name)
        .watch(true)
        .build()?;

    ctrl.open_input_device(&name)?;

    let output = ctrl.kill_for_output()?;

    assert!(output
        .stdout
        .contains("warning: No device was selected, but --watch is waiting for new devices."));

    ctrl.kill()
}

#[test]
pub fn e2e_disconnecting_device_in_watch_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder().watch(true).build()?;

    ctrl.close_input_device()?;

    let output = ctrl.kill_for_output()?;

    assert!(output.stdout.contains("Found a removed device. Reselecting devices."));

    Ok(())
}

#[test]
pub fn e2e_disconnecting_two_devices_in_watch_mode() -> anyhow::Result<()> {
    let name = get_random_device_name();
    let name2 = format!("{name} 2");

    let mut ctrl = {
        let _dev1 = get_virtual_device(&name)?;
        let _dev2 = get_virtual_device(&name2)?;

        let ctrl = XremapController::builder()
            .custom_input_device(&name)
            .watch(true)
            .build()?;

        ctrl

        // Devices will now be dropped
    };

    let output = ctrl.kill_for_output()?;

    assert!(output.stdout.contains("Found a removed device. Reselecting devices."));

    Ok(())
}
