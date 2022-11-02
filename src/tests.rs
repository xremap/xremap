use evdev::Key;
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::time::Duration;

use crate::{
    action::Action,
    config::Config,
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
        vec![Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    )
}

fn assert_actions(config_yaml: &str, events: Vec<Event>, actions: Vec<Action>) {
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let config: Config = serde_yaml::from_str(config_yaml).unwrap();
    let mut event_handler = EventHandler::new(timer, "default", Duration::from_micros(0));
    let mut actual: Vec<Action> = vec![];
    for event in &events {
        actual.append(&mut event_handler.on_event(event, &config).unwrap());
    }
    assert_eq!(format!("{:?}", actions), format!("{:?}", actual));
}
