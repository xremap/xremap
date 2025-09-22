use crate::action::Action;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::{assert_actions, get_input_device_info};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_virtual_modifier_is_never_emitted() {
    assert_actions(
        indoc! {"
        virtual_modifiers:
            - CAPSLOCK

        keymap:
            - remap:
                CAPSLOCK-A: B
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
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

#[test]
fn test_virtual_modifier_in_inexact_match() {
    assert_actions(
        indoc! {"
        virtual_modifiers:
            - CAPSLOCK
        keymap:
            - remap:
                CONTROL-A: B
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
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
fn test_ordinary_modifier_as_virtual() {
    assert_actions(
        indoc! {"
        virtual_modifiers:
            - WIN_L

        keymap:
            - remap:
                WIN_L-A: B
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            // Bug: this must not happen, because WIN_L is not released again.
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_modifier_not_declared() {
    // Wouldn't it be better to give a warning here, telling the mapping has no effect?
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                ENTER-A: B
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_ENTER, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_ENTER, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_ENTER, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_ENTER, KeyValue::Release)),
        ],
    )
}


#[test]
fn test_modmap_output_is_used_in_virtual_modifiers() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                Shift_L: Capslock
        virtual_modifiers:
            - Capslock
        keymap:
            - remap:
                Capslock-A: B
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
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
