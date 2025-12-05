#![cfg(feature = "device-test")]

use crate::common::xremap_controller::{InputDeviceFilter, XremapController};
use crate::common::{assert_err, assert_err_contains, assert_str_contains};
use anyhow::Result;
use indoc::indoc;

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
        .input_device(InputDeviceFilter::CustomFilter {
            filter: "match_nothing".into(),
        })
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

#[test]
pub fn e2e_error_in_config_file() -> Result<()> {
    let ctrl = XremapController::builder()
        .config(indoc! {"
                keymap:
                    - remap:
                        CustomKey: asdf
                "})?
        .input_device(InputDeviceFilter::NoFilter)
        .not_open_for_fetch()
        .build()?;

    let output = ctrl.wait_for_output()?;

    assert_str_contains(
        "Failed to load config '/tmp/xremap_config.yml': keymap[0].remap: unknown key 'CustomKey' at line 3 column 9",
        &output.stderr,
    );

    Ok(())
}
