#![cfg(test)]

use evdev::KeyCode as Key;
use evdev::{
    uinput::VirtualDevice as UInputDevice, AttributeSet, Device as EvDevice, EventType, InputEvent, SynchronizationCode,
};
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::action::Action;
use crate::action_dispatcher::ActionDispatcher;
use crate::client::{Client, WMClient};
use crate::device::{get_input_devices, output_device, InputDevice, InputDeviceInfo};
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::EventHandler;
use crate::{config::keymap::build_keymap_table, Config};

struct StaticClient {
    current_application: Option<String>,
}

#[test]
#[ignore]
fn test_pipeline_passthrough_manual_virtual_input() {
    let vin_name = format!("xremap-test-input-{}", std::process::id());
    let mut keys: AttributeSet<Key> = AttributeSet::new();
    keys.insert(Key::KEY_A);
    let mut vinput = UInputDevice::builder()
        .unwrap()
        .name(&vin_name)
        .with_keys(&keys)
        .unwrap()
        .build()
        .unwrap();
    println!("[pipeline] Virtual INPUT created: {vin_name}");

    let mut inputs = get_input_devices(&vec![vin_name.clone()], &vec![], false, false).unwrap();
    let (in_path, input_dev): (PathBuf, InputDevice) = inputs.into_iter().next().unwrap();
    let mut input_dev = input_dev;
    println!("[pipeline] Registered INPUT at {}", in_path.display());

    let mut config: Config = serde_yaml::from_str("").unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut handler = EventHandler::new(
        timer,
        "default",
        Duration::from_micros(0),
        WMClient::new(
            "static",
            Box::new(StaticClient {
                current_application: None,
            }),
        ),
    );

    let vout = output_device(None, /*enable_wheel=*/ false, 0x1234, 0x5678).unwrap();
    let mut dispatcher = ActionDispatcher::new(vout);
    println!("[pipeline] Virtual OUTPUT created (name contains 'xremap' or 'xremap pid=PID')");

    let pid_name = format!("xremap pid={}", std::process::id());
    let out_event_path = find_event_device_by_name(&pid_name, 2000)
        .or_else(|| find_event_device_by_name("xremap", 2000))
        .unwrap();
    println!("[pipeline] OUTPUT event device: {}", out_event_path.display());
    let mut out_dev = EvDevice::open(&out_event_path).unwrap();

    vinput
        .emit(&[
            InputEvent::new_now(EventType::KEY.0, Key::KEY_A.code(), 1),
            InputEvent::new_now(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0),
        ])
        .unwrap();
    vinput
        .emit(&[
            InputEvent::new_now(EventType::KEY.0, Key::KEY_A.code(), 0),
            InputEvent::new_now(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0),
        ])
        .unwrap();
    println!("[pipeline] Emitted KEY_A press/release to INPUT");

    let events: Vec<_> = input_dev.fetch_events().unwrap().collect();
    let mut ev_wrapped = Vec::with_capacity(events.len());
    for e in events {
        ev_wrapped.push(Event::new(input_dev.to_info(), e));
    }
    let actions = handler.on_events(&ev_wrapped, &config).unwrap();
    for a in actions {
        dispatcher.on_action(a).unwrap();
    }

    let deadline = std::time::Instant::now() + Duration::from_millis(500);
    let mut saw_press = false;
    let mut saw_release = false;
    while std::time::Instant::now() < deadline && !(saw_press && saw_release) {
        if let Ok(iter) = out_dev.fetch_events() {
            for e in iter {
                if e.event_type() == EventType::KEY && e.code() == Key::KEY_A.code() {
                    if e.value() == 1 {
                        saw_press = true;
                    }
                    if e.value() == 0 {
                        saw_release = true;
                    }
                }
            }
        }
        if !(saw_press && saw_release) {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    assert!(saw_press, "expected KEY_A press in OUTPUT");
    assert!(saw_release, "expected KEY_A release in OUTPUT");

    input_dev.ungrab();
    println!("[test_pipeline_passthrough_manual_virtual_input] DONE");
}

fn find_event_device_by_name(name_substr: &str, timeout_ms: u64) -> Option<PathBuf> {
    let start = std::time::Instant::now();
    loop {
        if let Ok(dir) = read_dir("/dev/input") {
            for entry in dir.flatten() {
                let path = entry.path();
                if !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .starts_with("event")
                {
                    continue;
                }
                if let Ok(dev) = EvDevice::open(&path) {
                    let name = dev.name().unwrap_or("");
                    if name.contains(name_substr) {
                        return Some(path);
                    }
                }
            }
        }
        if start.elapsed() > Duration::from_millis(timeout_ms) {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore]
fn test_emit_via_virtual_keyboard_manual() {
    println!("\n[test_emit_via_virtual_keyboard_manual] START");

    let mut config: Config = serde_yaml::from_str("").unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut handler = EventHandler::new(
        timer,
        "default",
        Duration::from_micros(0),
        WMClient::new(
            "static",
            Box::new(StaticClient {
                current_application: None,
            }),
        ),
    );
    println!("[test_emit_via_virtual_keyboard_manual] Handler ready");

    let vdev = output_device(None, /*enable_wheel=*/ false, 0x1234, 0x5678).unwrap();
    let mut dispatcher = ActionDispatcher::new(vdev);
    println!("[test_emit_via_virtual_keyboard_manual] Virtual keyboard created");

    let events = vec![
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
    ];
    println!("[test_emit_via_virtual_keyboard_manual] Input events:");
    for e in &events {
        println!("  {:?}", e);
    }
    let actions = handler.on_events(&events, &config).unwrap();
    println!("[test_emit_via_virtual_keyboard_manual] Dispatching actions:");
    for action in actions {
        println!("  {:?}", action);
        dispatcher.on_action(action).unwrap();
    }

    std::thread::sleep(Duration::from_millis(10));
    println!("[test_emit_via_virtual_keyboard_manual] DONE");
}

impl Client for StaticClient {
    fn supported(&mut self) -> bool {
        true
    }
    fn current_window(&mut self) -> Option<String> {
        None
    }
    fn current_application(&mut self) -> Option<String> {
        self.current_application.clone()
    }
}

fn get_input_device_info<'a>() -> InputDeviceInfo<'a> {
    InputDeviceInfo {
        name: "Some Device",
        path: Path::new("/dev/input/event0"),
        vendor: 0x1234,
        product: 0x5678,
    }
}

fn run_events(events: Vec<Event>) -> Vec<Action> {
    println!("\n[run_events] Incoming events:");
    for e in &events {
        println!("  {:?}", e);
    }

    let mut config: Config = serde_yaml::from_str("").unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);

    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut handler = EventHandler::new(
        timer,
        "default",
        Duration::from_micros(0),
        WMClient::new(
            "static",
            Box::new(StaticClient {
                current_application: None,
            }),
        ),
    );

    let actions = handler.on_events(&events, &config).unwrap();
    println!("[run_events] Produced actions:");
    for a in &actions {
        println!("  {:?}", a);
    }
    actions
}

#[test]
fn test_e2e_sequence_phases_basic() {
    let events = vec![
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFT, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFT, KeyValue::Release)),
    ];

    let expected = vec![
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFT, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFT, KeyValue::Release)),
    ];

    let actual = run_events(events);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[test]
fn test_e2e_interleaved_modifier_and_taps() {
    let events = vec![
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
    ];

    let expected = vec![
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
    ];

    let actual = run_events(events);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[test]
fn test_e2e_repeat_passthrough() {
    let events = vec![
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
    ];

    let expected = vec![
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
    ];

    let actual = run_events(events);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[test]
fn test_e2e_hold_then_interleaved_release() {
    let events = vec![
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
    ];

    let expected = vec![
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
    ];

    let actual = run_events(events);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}
