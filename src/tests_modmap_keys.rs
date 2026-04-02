use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_modmap_one_key() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: SHIFT_L
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_remap_two_concurrent_keys() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: SHIFT_L
              CTRL_L: ALT_L
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_only_emits_press_on_press() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: SHIFT_L
        "},
        vec![Event::key_press(Key::KEY_CAPSLOCK)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press))],
    );
}

#[test]
fn test_modmap_can_emit_several_keys() {
    // Note that modifiers are not sorted first/last as the multipurpose keys are.
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: [SHIFT_L, V]
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_V, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_V, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_followed_by_same_emit_key() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: SHIFT_L
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_release(Key::KEY_LEFTSHIFT),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_preceded_by_same_emit_key() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: SHIFT_L
        "},
        vec![
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_LEFTSHIFT),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_output_is_used_in_keymap() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              CAPSLOCK: X
        keymap:
          - remap:
              X: KEY_1
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_modmap_emit_is_not_used_in_subsequent_remaps() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                a: b
            - remap:
                b: c
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}
