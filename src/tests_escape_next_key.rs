use crate::action::Action;
use crate::event::{Event, RelativeEvent};
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::{KeyCode as Key, RelativeAxisCode};
use indoc::indoc;

#[test]
fn test_escape_next_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              esc: { escape_next_key: true }
              A: C
        "},
        vec![Event::key_press(Key::KEY_ESC), Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
    )
}

#[test]
fn test_release_does_not_cancel_escape_next_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              esc: { escape_next_key: true }
              A: C
        "},
        vec![
            Event::key_press(Key::KEY_K),
            Event::key_press(Key::KEY_ESC),
            Event::key_release(Key::KEY_K),
            Event::key_press(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_escape_next_disguised_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              esc: { escape_next_key: true }
              XUpScroll: C
        "},
        vec![
            Event::key_press(Key::KEY_ESC),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 10),
        ],
        vec![Action::RelativeEvent(RelativeEvent::new_with(
            RelativeAxisCode::REL_WHEEL.0,
            10,
        ))],
    )
}

#[test]
fn test_modifier_does_not_affect_escape_next_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              esc: { escape_next_key: true }
              Ctrl_L: A
              Ctrl_L-B: C
        "},
        vec![
            Event::key_press(Key::KEY_ESC),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_B),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_virtual_modifier_does_not_affect_escape_next_key() {
    assert_actions(
        indoc! {"
        virtual_modifiers: [Capslock]
        keymap:
          - remap:
              esc: { escape_next_key: true }
              Capslock: A
              Capslock-B: C
        "},
        vec![
            Event::key_press(Key::KEY_ESC),
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_B),
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    )
}

#[test]
fn test_matching_any_key_does_not_affect_escape_next_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              esc: { escape_next_key: true }
              ANY: A
              c_l-B: C
        "},
        vec![
            Event::key_press(Key::KEY_ESC),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_B),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
        ],
    )
}
