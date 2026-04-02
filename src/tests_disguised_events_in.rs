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
fn test_disguised_event_does_not_match() {
    assert_actions(
        indoc! {""},
        vec![Event::relative(RelativeAxisCode::REL_WHEEL.0, 1)],
        vec![Action::RelativeEvent(RelativeEvent::new_with(
            RelativeAxisCode::REL_WHEEL.0,
            1,
        ))],
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
fn test_disguised_event_mapped_in_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    XUpScroll: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 1),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_disguised_event_cancels_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a:
                remap:
                    b: c
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 1),
            Event::key_press(Key::KEY_B),
        ],
        vec![
            Action::RelativeEvent(RelativeEvent::new_with(RelativeAxisCode::REL_WHEEL.0, 1)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_mouse_wheel_cancels_multi_purpose_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                a:
                    alone: b
                    held: c
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::relative(RelativeAxisCode::REL_WHEEL.0, 1),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::RelativeEvent(RelativeEvent::new_with(RelativeAxisCode::REL_WHEEL.0, 1)),
        ],
    )
}

#[test]
fn test_mouse_movement_does_not_cancel_multi_purpose_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                a:
                    alone: b
                    held: c
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::relative(RelativeAxisCode::REL_X.0, 1),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            // It's not expected, that mouse movement happens before, but is emitted last.
            // Might never be a problem, because keys and mouse movement is probably never
            // emitted by same device (in same batch).
            // To fix it, mouse movement must be batched according to synchronization events,
            // before it's sent into EventHandler.
            Action::MouseMovementEventCollection(vec![RelativeEvent::new_with(RelativeAxisCode::REL_X.0, 1)]),
        ],
    )
}

#[test]
fn test_opt_in_mouse_movement_interrupts_multi_purpose_key() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                a:
                    alone: b
                    held: c
                    interruptable: true
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::relative(RelativeAxisCode::REL_X.0, 1),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::MouseMovementEventCollection(vec![RelativeEvent::new_with(RelativeAxisCode::REL_X.0, 1)]),
        ],
    )
}
