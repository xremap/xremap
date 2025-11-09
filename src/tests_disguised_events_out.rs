use crate::action::Action;
use crate::event::Event;
use crate::event::RelativeEvent;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use evdev::RelativeAxisCode;
use indoc::indoc;
use std::time::Duration;

// Emitting disguised keys (i.e. mouse events)
// Only some use cases work.

#[test]
fn test_emit_disguised_events_with_press_release_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A:
                    press: XUpScroll
                    release: XDownScroll
                    skip_key_event: true
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key(59990), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59990), KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_disguised_event_trigger_same_disguised_event_in_modmap() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                XUpScroll: [C_L, XUpScroll, ALT_L]
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::RelativeEvent(RelativeEvent::new_with(RelativeAxisCode::REL_WHEEL.0, 1)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_disguised_event_trigger_same_disguised_event_in_keymap() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                XUpScroll: C_L-XUpScroll
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59990), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59990), KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_disguised_event_trigger_other_disguised_event_in_modmap() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                XUpScroll: [C_L, XDownScroll, ALT_L]
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_disguised_event_trigger_other_disguised_event_in_keymap() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                XUpScroll: C_L-XDownScroll
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_key_trigger_disguised_event_in_modmap() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A: XDownScroll
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Release)),
        ],
    )
}

#[test]
fn test_key_trigger_disguised_event_in_keymap() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                A: C_L-XDownScroll
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key(59991), KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}
