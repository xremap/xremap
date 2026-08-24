use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::tests::{assert_actions, EventHandlerForTest};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_emacs_like() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                C-space: { set_mark: true }
                C-f: { with_mark: right }
                C-g: [esc, { set_mark: false }]
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_SPACE),
            Event::key_press(Key::KEY_F),
            // Remove mark again
            Event::key_press(Key::KEY_G),
            // Now it's without shift
            Event::key_press(Key::KEY_F),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_ESC, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_ESC, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_mark_triggered_by_shift_combo() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        keymap:
            - remap:
                f12: { set_mark: true }
                A: { with_mark: '1' }
                S-B: { with_mark: '2' }
        "});

    handler.assert(vec![Event::key_press(Key::KEY_F12)], vec![]);

    handler.assert(
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_LEFTSHIFT), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_mark_triggered_by_shift_combo_and_emits_shift_combo() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        keymap:
            - remap:
                f12: { set_mark: true }
                A: { with_mark: S-1 }
                S-B: { with_mark: S-2 }
        "});

    handler.assert(vec![Event::key_press(Key::KEY_F12)], vec![]);

    handler.assert(
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_LEFTSHIFT), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_mark_triggered_with_extra_modifiers() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        keymap:
            - remap:
                f12: { set_mark: true }
                A: { with_mark: '1' }
                S-B: { with_mark: '2' }
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_LEFTCTRL)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press))],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_LEFTSHIFT), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}

#[test]
fn test_mark_triggered_with_extra_modifiers_and_emits_shift_combo() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        keymap:
            - remap:
                f12: { set_mark: true }
                A: { with_mark: S_L-1 }
                S-B: { with_mark: S_R-2 }
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_LEFTCTRL)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press))],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    handler.assert(
        vec![Event::key_press(Key::KEY_LEFTSHIFT), Event::key_press(Key::KEY_B)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_2, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHTSHIFT, KeyValue::Release)),
        ],
    );
}
