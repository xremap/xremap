use crate::action::Action;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_press_release() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A:
                    press: C
        "},
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_press_release_skip_original_key() {
    //Note: press release key has no repeat/release actions by default
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A:
                    press: C
                    skip_key_event: true
        "},
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_press_release_repeat_original_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A:
                    press: C
                    release: D
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_repeat(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_press_release_repeat_custom_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                A:
                    press: C
                    repeat: E
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_repeat(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_E, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_E, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_press_release_actions_goes_around_keymap() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              A:
                press: B
                skip_key_event: true

        keymap:
          - remap:
              B: C
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_press_release_can_escape_next_key() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              A:
                press: B
                release: { escape_next_key: true }
                skip_key_event: true

        keymap:
          - remap:
              C: D
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_press_release_emit_modifiers() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              A:
                press: LeftCtrl-B
                skip_key_event: true
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_press_release_lift_extra_modifiers_on_emit() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              A:
                press: LeftCtrl-B
                skip_key_event: true
        "},
        vec![
            Event::key_press(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}
