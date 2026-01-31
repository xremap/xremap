#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use crate::common::{assert_str_contains, get_random_device_name, get_virtual_device};

mod common;

#[test]
pub fn e2e_match_nothing_in_watch_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
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
        .input_device(InputDeviceFilter::CustomFilter { filter: name.clone() })
        .watch(true)
        .build()?;

    ctrl.open_input_device(&name)?;

    let output = ctrl.kill_for_output()?;

    assert_str_contains("warning: No device was selected, but --watch is waiting for new devices.", &output.stdout);
    // A poor test, that device is now selected.
    assert_str_contains(&format!(": {}", name), &output.stdout);

    ctrl.kill()
}

#[test]
pub fn e2e_disconnecting_device_in_watch_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder().watch(true).build()?;

    ctrl.close_input_device()?;

    // Give time to handle the event, otherwise it's a race-condition whether
    //  kill or handling-disconnect happens first.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let output = ctrl.kill_for_output()?;

    assert_str_contains("Found a removed device. Reselecting devices.", &output.stdout);

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
            .input_device(InputDeviceFilter::CustomFilter { filter: name.clone() })
            .watch(true)
            .build()?;

        ctrl

        // Devices will now be dropped
    };

    // Give time to handle the event, otherwise it's a race-condition whether
    //  kill or handling-disconnect happens first.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let output = ctrl.kill_for_output()?;

    assert!(output.stdout.contains("Found a removed device. Reselecting devices."));

    Ok(())
}
