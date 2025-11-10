use crate::action::Action;
use crate::event::Event;
use crate::event::RelativeEvent;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use evdev::RelativeAxisCode;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_mapped_disguised_event_from_modmap_is_used_in_keymap() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                XUpScroll: C
        keymap:
            - remap:
                C: D
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_relative_events_in_keymap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              XRightCursor: b
        "},
        vec![Event::relative(RelativeAxisCode::REL_X.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_relative_events_in_keymap_with_held_modifier() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              Ctrl-XRightCursor: b
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::relative(RelativeAxisCode::REL_X.0, 1),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_mixed_mouse_events_matching_and_non_matching() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              XRightCursor: b
        "},
        vec![
            Event::relative(RelativeAxisCode::REL_X.0, 10),
            Event::relative(RelativeAxisCode::REL_Y.0, -10),
            Event::relative(RelativeAxisCode::REL_X.0, 100),
            Event::relative(RelativeAxisCode::REL_Y.0, -100),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::MouseMovementEventCollection(vec![
                RelativeEvent::new_with(RelativeAxisCode::REL_Y.0, -10),
                RelativeEvent::new_with(RelativeAxisCode::REL_Y.0, -100),
            ]),
        ],
    )
}

#[test]
fn test_mixed_wheel_events_matching_and_non_matching() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              XUpScroll: b
        "},
        vec![
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 10),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, -10),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 100),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, -100),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::RelativeEvent(RelativeEvent::new_with(RelativeAxisCode::REL_WHEEL.0, -10)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::RelativeEvent(RelativeEvent::new_with(RelativeAxisCode::REL_WHEEL.0, -100)),
        ],
    )
}

#[test]
fn test_disguised_events_and_multipurpose_key() {
    // This use case is of little use, because the release is fired immediately.
    // so the alone definition is always emitted.

    assert_actions(
        indoc! {"
        modmap:
            - remap:
                XUpScroll:
                    alone: c
                    held: d
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_disguised_events_and_press_release_key() {
    // This use case is of little use, because scroll emits press and release immediately
    // so they could be joint into just a press-action
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                XUpScroll:
                    press: C
                    release: D
                    skip_key_event: true
        "},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}
