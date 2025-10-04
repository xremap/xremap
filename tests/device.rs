#![cfg(feature = "device-test")]

use xremap::device::get_input_devices;

use crate::common::assert_err;

mod common;

#[test]
pub fn test_no_input_device_match() {
    let device_filter = vec!["match_nothing".into()];

    assert_err("No device was selected!", get_input_devices(&device_filter, &vec![], false, false));
}

#[test]
pub fn test_device_filter_overwrites_keyboard_and_mouse_check() {
    let device_filter = vec!["Power Button".into()];

    assert!(get_input_devices(&device_filter, &vec![], false, false).is_ok());
}
