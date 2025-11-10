use crate::action::Action;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_merge_nested_sibling_remaps() {
    let config = indoc! {"
        keymap:
          - remap:
              capslock:
                - remap:
                    a: b
                - remap:
                    c: d
    "};

    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_merge_nested_sibling_remaps_precedence_to_first() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              CAPSLOCK:
                - remap:
                    a: b
                - remap:
                    a: c
    "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_event_canceling_remap_gets_emitted() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_the_event_canceling_remap_gets_emitted_when_same_as_trigger_key_when_implicit() {
    // This does not work
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_the_event_canceling_remap_gets_emitted_when_same_as_trigger_key_when_explicit() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
                    a: a
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_modifier_canceling_remap_gets_emitted() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_event_canceling_remap_is_used_for_matching() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
              k: l
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_L, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_L, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_modifier_canceling_remap_is_used_for_matching() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
              ctrl-k: l
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_L, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_L, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_cancel_by_timeout_emits_nothing() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::OverrideTimeout,
            // Check it's canceled.
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_cancel_by_timeout_with_timeout_key() {
    // There is no default timeout_millis so timeout_key is just ignored.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                timeout_key: t
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::OverrideTimeout,
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_cancel_by_timeout_with_explicit_timeout() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                timeout_key: t
                timeout_millis: 100
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::OverrideTimeout,
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_T, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_T, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_cancel_by_key_with_defined_timeout_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                timeout_key: t
                timeout_millis: 100
                remap:
                    c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_T, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_T, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_cancel_by_key_with_defined_timeout_key_but_no_match() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                timeout_key: t
                timeout_millis: 100
                remap:
                    ctrl-c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            // KEY_C doesn't match because it has the wrong modifiers
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}
