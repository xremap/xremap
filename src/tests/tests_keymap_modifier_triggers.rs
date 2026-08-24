use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_modifier_triggers_alone() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              c_l: end
        "},
        vec![Event::key_press(Key::KEY_LEFTCTRL)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_modifier_triggers_modded() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              c_l-c_r: end
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_RIGHTCTRL),
            Event::key_repeat(Key::KEY_RIGHTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            // Repeat
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_END, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_modifier_trigger_cannot_be_its_own_modifier() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              c-c_r: end
        "},
        vec![
            Event::key_press(Key::KEY_RIGHTCTRL),
            Event::key_repeat(Key::KEY_RIGHTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_modifier_trigger_can_be_used_as_normal_modifier() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
                c-c_r: A
                c_r-B: C
        "},
        vec![Event::key_press(Key::KEY_RIGHTCTRL), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_modifier_trigger_that_match_can_be_used_as_normal_modifier() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
                c_r: A
                c_r-B: C
        "},
        vec![Event::key_press(Key::KEY_RIGHTCTRL), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            // Second match
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_modifier_trigger_sends_other_modifier_combo() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              alt_l-alt_r: c-x
        "},
        vec![Event::key_press(Key::KEY_LEFTALT), Event::key_press(Key::KEY_RIGHTALT)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTALT, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_modifier_trigger_sends_same_modifier_combo() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              c_r-c_l: c-x
        "},
        vec![
            Event::key_press(Key::KEY_RIGHTCTRL),
            Event::key_press(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_modifier_trigger_with_extra_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              shift_r: A
        "},
        vec![
            Event::key_press(Key::KEY_LEFTMETA),
            Event::key_press(Key::KEY_RIGHTSHIFT),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_modifier_trigger_with_exact_match() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              shift_r: A
              win_l-shift_r: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTMETA),
            Event::key_press(Key::KEY_RIGHTSHIFT),
            Event::key_repeat(Key::KEY_RIGHTSHIFT),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            // Repeat
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_modifier_trigger_with_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              s_r:
                remap:
                  A: B
        "},
        vec![Event::key_press(Key::KEY_RIGHTSHIFT), Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_modifier_trigger_with_nested_remap_cancelled() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              s_r:
                remap:
                  A: B
        "},
        vec![Event::key_press(Key::KEY_RIGHTSHIFT), Event::key_press(Key::KEY_K)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_modifier_trigger_with_nested_remap_timeout() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              s_r:
                timeout_millis: 100
                remap:
                  A: B
        "},
        vec![
            Event::key_press(Key::KEY_RIGHTSHIFT),
            Event::OverrideTimeout,
            Event::key_press(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    )
}
