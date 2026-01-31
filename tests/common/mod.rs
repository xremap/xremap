#![allow(dead_code)]

pub mod xremap_controller;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, BusType, Device, EventType, InputEvent, InputId, KeyCode, SwitchCode};
use std::iter::repeat_with;
use std::path::PathBuf;
use std::time::Duration;
use xremap::util::{until, until_value};

pub fn key_click(key: KeyCode) -> Vec<InputEvent> {
    vec![key_press(key), key_release(key)]
}

pub fn key_release(key: KeyCode) -> InputEvent {
    InputEvent::new(EventType::KEY.0, key.code(), 0)
}

pub fn key_press(key: KeyCode) -> InputEvent {
    InputEvent::new(EventType::KEY.0, key.code(), 1)
}

pub fn get_raw_device_pair() -> anyhow::Result<(Device, VirtualDeviceInfo)> {
    let dev_info = get_virtual_device(get_random_device_name())?;

    let mut input = Device::open(&dev_info.path)?;

    input.grab()?;

    Ok((input, dev_info))
}

pub struct VirtualDeviceInfo {
    pub name: String,
    pub path: PathBuf,
    pub device: VirtualDevice,
}

pub fn get_random_device_name() -> String {
    format!("test device {}", repeat_with(fastrand::alphanumeric).take(10).collect::<String>())
}

pub fn get_virtual_device(name: impl Into<String>) -> anyhow::Result<VirtualDeviceInfo> {
    let name = name.into();

    let mut keys: AttributeSet<KeyCode> = AttributeSet::new();
    for code in KeyCode::KEY_RESERVED.code()..KeyCode::BTN_TRIGGER_HAPPY40.code() {
        let key = KeyCode::new(code);
        let name = format!("{:?}", key);
        if name.starts_with("KEY_") || name.starts_with("BTN_") {
            keys.insert(key);
        }
    }

    let mut sw: AttributeSet<SwitchCode> = AttributeSet::new();

    sw.insert(SwitchCode::SW_LID);
    sw.insert(SwitchCode::SW_TABLET_MODE);

    let device = VirtualDevice::builder()?
        .input_id(InputId::new(BusType::BUS_USB, 0x1234, 0x5678, 0x111))
        .name(&name)
        .with_keys(&keys)?
        .with_switches(&sw)?
        .build()?;

    // Fetch path.
    let (path, _) = wait_for_device(&name)?;

    Ok(VirtualDeviceInfo { name, path, device })
}

pub fn wait_for_device(name: &str) -> anyhow::Result<(PathBuf, Device)> {
    until_value(
        || evdev::enumerate().find(|(_, device)| name == device.name().unwrap_or_default()),
        Duration::from_secs(1),
        &format!("Timed out waiting for device: {name}"),
    )
}

// Wait for the device to be grabbed by some other process.
pub fn wait_for_grabbed(path: &PathBuf) -> anyhow::Result<()> {
    until(
        || {
            let mut probe_device = Device::open(&path).unwrap();

            if probe_device.grab().is_err() {
                true
            } else {
                probe_device.ungrab().unwrap();
                false
            }
        },
        Duration::from_secs(1),
        &format!("Timed out waiting for device to be grabbed: {path:?}"),
    )
}

pub fn final_event_state(key: KeyCode, events: impl IntoIterator<Item = InputEvent>) -> Option<i32> {
    events.into_iter().fold(None, |state, ev| {
        if ev.event_type() == EventType::KEY && ev.code() == key.code() {
            if ev.value() == 0 {
                Some(0)
            } else {
                Some(1)
            }
        } else {
            state
        }
    })
}

pub fn assert_err<T, E>(expected: &str, result: Result<T, E>)
where
    E: ToString,
{
    match result {
        Ok(_) => panic!("\nExpected an error.\n"),
        Err(e) => {
            assert_eq!(expected, e.to_string());
        }
    }
}

pub fn assert_err_contains<T, E>(expected: &str, result: Result<T, E>)
where
    E: ToString,
{
    match result {
        Ok(_) => panic!("\nExpected an error.\n"),
        Err(e) => {
            if !e.to_string().contains(expected) {
                panic!("Should contain: {expected}\nError: {}", e.to_string());
            }
        }
    }
}

pub fn assert_str_contains(expected: &str, str: &str) {
    if !str.to_string().contains(expected) {
        panic!("Should contain: {expected}\n{str}");
    }
}
