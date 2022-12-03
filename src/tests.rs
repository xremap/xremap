use evdev::Key;
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::time::Duration;

use crate::{
    action::Action,
    config::{keymap::build_keymap_table, Config},
    event::{Event, KeyEvent, KeyValue},
    event_handler::EventHandler,
};

#[test]
fn test_basic_modmap() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              a: b
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_interleave_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

fn assert_actions(config_yaml: &str, events: Vec<Event>, actions: Vec<Action>) {
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut config: Config = serde_yaml::from_str(config_yaml).unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    let mut event_handler = EventHandler::new(timer, "default", Duration::from_micros(0));
    let mut actual: Vec<Action> = vec![];
    for event in &events {
        actual.append(&mut event_handler.on_event(event, &config).unwrap());
    }
    assert_eq!(format!("{:?}", actions), format!("{:?}", actual));
}
