use crate::action::Action;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::{assert_actions, get_input_device_info};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_multipurpose_is_interrupted() {
    // Held key is emitted if another key is pressed before release
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_multipurpose_released_without_interuption() {
    // Alone key is emitted if no other key is pressed before release
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_multipurpose_press_all_keys_then_release_all_keys_1() {
    // Held key will emit all key presses of the held-property before the triggering key is emitted.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: [X, Y]
                    alone: A
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_multipurpose_press_all_keys_then_release_all_keys_2() {
    // Held key will emit all key presses of the held-property before the two triggering keys are emitted.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: [X, Y]
                    alone: A
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_multipurpose_press_all_keys_then_release_all_keys_3() {
    // Alone key will emit all key presses before any release.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: X
                    alone: [A, B]
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_key_release_will_not_trigger_held() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                held: X
                alone: Y
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_the_multipurpose_output_is_used_in_keymap_1() {
    // If key is held and a match in keymap is hit, then the keymap-action is emitted followed by the interrupting key.
    // It has to be that way because keymap emits press/release already when the key is pressed.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: X
                    alone: Y
                    free_hold: true
        keymap:
            - remap:
                X: KEY_1
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_the_multipurpose_output_is_used_in_keymap_2() {
    // If key is alone and a match in keymap is hit then the keymap is used.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: X
                    alone: Y
                    free_hold: true
        keymap:
            - remap:
                Y: KEY_1
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_the_multipurpose_output_is_used_in_keymap_3() {
    // If key is alone while a non-modifier was already pressed.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: X
                    alone: Y
                    free_hold: true
        keymap:
            - remap:
                Y: KEY_1
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_the_multipurpose_output_is_used_in_keymap_4() {
    // Alone while a modifier was already pressed
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: X
                    alone: Y
                    free_hold: true
        keymap:
            - remap:
                Y: KEY_1
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_alone_is_not_optional() {
    // The configuration is just ignored. A warning would be better.
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                held: X
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_held_is_not_optional() {
    // The configuration is just ignored. A warning would be better.
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                alone: X
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_output_key_used_as_trigger() {
    // A press is filtered from the output. Is there a reason for this. In which use case is it needed?
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                held: X
                alone: Y
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modifiers_are_sorted_first_and_last_held() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                held: [X, Control_L, Y]
                alone: A
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modifiers_are_sorted_first_and_last_alone() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK:
                held: A
                alone: [X, Control_L, Y]
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Y, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}
