use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

/// Tests the logic of extra_modifiers property in event_dispatcher
/// This property ensures that extra modifiers that are pressed will
/// be released when the mapping is emitted.

#[test]
fn test_on_left_side() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                CONTROL-A: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_on_right_side() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                A: CONTROL-B
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_modifier_not_released_in_inexact_match() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                A: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_virtual_modifier_is_not_considered_extra() {
    assert_actions(
        indoc! {"
        virtual_modifiers:
            - CAPSLOCK
        keymap:
            - remap:
                CAPSLOCK-A: B
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}
