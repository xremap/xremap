#![cfg(feature = "device-test")]

use crate::common::{assert_err, assert_err_contains, xremap_controller::XremapController};

mod common;

#[test]
pub fn e2e_keeps_running() -> anyhow::Result<()> {
    // Xremap is started correctly so it won't stop by itself.
    let ctrl = XremapController::new()?;

    assert_err("Timed out waiting for xremap exit", ctrl.wait_for_output());

    ctrl.kill()
}

#[test]
pub fn e2e_get_device_that_xremap_never_opens() -> anyhow::Result<()> {
    // Fails and exits without opening a VirtualDevice
    let mut ctrl = XremapController::builder()
        .custom_input_device("match_nothing")
        .not_open_for_fetch()
        .build()?;

    assert_err_contains("Timed out waiting for device: test output device", ctrl.open_output_device());

    ctrl.kill()
}

#[test]
pub fn e2e_emit_to_device_that_is_not_grabbed() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    ctrl.kill_for_output()?;

    assert_err("Input device not grabbed.", ctrl.emit_events(&vec![]));

    Ok(())
}

#[test]
pub fn e2e_timeout_waiting_for_events() -> anyhow::Result<()> {
    let mut ctrl = XremapController::new()?;

    assert_err("Timed out waiting for xremap events.", ctrl.fetch_events());

    ctrl.kill()
}

#[test]
pub fn e2e_wait_for_output_with_nocapture() -> anyhow::Result<()> {
    let ctrl = XremapController::builder().nocapture().build()?;

    assert_err("Can't get output when configured for nocapture.", ctrl.wait_for_output());

    ctrl.kill()
}
