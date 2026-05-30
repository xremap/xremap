use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::{KeyCode as Key, RelativeAxisCode};
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_any_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a: b
              ANY: null
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
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_disguised_keys_match_any_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              ANY: A
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 10)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}

#[test]
fn test_anykey_can_activate_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              ANY:
                remap:
                  A: B
        "},
        vec![Event::key_press(Key::KEY_K), Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}

#[test]
fn test_any_does_not_work_in_nested_remap() {
    // This happens because it tries nested with K, and then cancels the nested
    // remap because K is not mapped there. So it doesn't matter that it's later
    // tried with any-key.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                - remap:
                    b: c
                    any: d
              k: h
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_K)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}
