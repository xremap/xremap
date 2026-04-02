use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::{assert_actions, EventHandlerForTest};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_tap_preferred_is_not_emitted_on_press() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: A
                    alone: B
                    hold_threshold_millis: 1000
        "},
        vec![Event::key_press(Key::KEY_CAPSLOCK)],
        vec![],
    );
}

#[test]
fn test_tap_preferred_released_before_timeout() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 1000
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_tap_preferred_interrupted_before_timeout() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 1000
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_B),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_B),
            Event::key_release(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_tap_preferred_interrupted_before_timeout_and_repeated() {
    // Repeat is ignored.
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 1000
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_press(Key::KEY_A),
            Event::key_repeat(Key::KEY_CAPSLOCK),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    );
}

#[test]
fn test_tap_preferred_is_not_repeated_before_timeout() {
    assert_actions(
        indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 1000
        "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_repeat(Key::KEY_CAPSLOCK),
        ],
        vec![],
    );
}

#[test]
fn test_tap_preferred_released_in_hold_preferred_state() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 10
                    tap_timeout: 200
        "});

    handler.assert(vec![Event::key_press(Key::KEY_CAPSLOCK)], vec![]);

    sleep(Duration::from_millis(20)); // To ensure in hold-preferred state.

    handler.assert(
        vec![Event::key_release(Key::KEY_CAPSLOCK)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_tap_preferred_interrupted_in_hold_preferred_state() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - remap:
                CAPSLOCK:
                    held: SHIFT_L
                    alone: CAPSLOCK
                    hold_threshold_millis: 10
                    tap_timeout: 200
        "});

    handler.assert(vec![Event::key_press(Key::KEY_CAPSLOCK)], vec![]);

    sleep(Duration::from_millis(20)); // To ensure in hold-preferred state.

    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    );
}
