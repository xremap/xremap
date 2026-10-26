use crate::action::Action;
use crate::event::{Event, KeyEvent};
use crate::tests::EventHandlerForTest;
use crate::KeyValue;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(10);

#[test]
fn test_expmap_bypass_double_tap() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        experimental_map:
            - bypass: true
              remap:
                A: { double: B}
        keymap:
            - remap:
                A: K
                B: K
        "});

    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(vec![Event::key_release(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_expmap_bypass_throttle() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        experimental_map:
            - bypass: true
              remap:
                A: { throttle_ms: 10 }
        keymap:
            - remap:
                A: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release))],
    );

    std::thread::sleep(TIMEOUT); // So it goes into done state
    handler.assert(
        vec![Event::key_press(Key::KEY_P), Event::key_release(Key::KEY_P)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_P, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_P, KeyValue::Release)),
        ],
    );

    handler.assert_base_state();
}

#[test]
fn test_expmap_bypass_oneshot() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        experimental_map:
            - bypass: true
              remap:
                A: { oneshot: B }
        keymap:
            - remap:
                A: K
                B: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    );
    handler.assert(vec![Event::key_release(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_C)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_C)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_expmap_bypass_select() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        experimental_map:
          - bypass: true
            remap:
              A: { select: [{ double: B }, { double: C }] }
        keymap:
            - remap:
                A: K
                B: K
                C: K
        "});

    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(vec![Event::key_release(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_expmap_bypass_chords() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        experimental_map:
          - bypass: true
            chords:
              - keys: [A, B]
                actions: C
        keymap:
            - remap:
                A: K
                B: K
                C: K
        "});

    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_B)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release))],
    );
    handler.assert(vec![Event::key_release(Key::KEY_B)], vec![]);

    handler.assert_base_state();
}

#[test]
fn test_global_bypass_used_in_expmap() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        bypass: true
        experimental_map:
            - remap:
                A: { double: B}
        keymap:
            - remap:
                A: K
                B: K
        "});

    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(vec![Event::key_release(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_global_bypass_overwritten_in_expmap() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        bypass: true
        experimental_map:
            - bypass: false
              remap:
                A: { double: B}
        keymap:
            - remap:
                A: C
                B: D
        "});

    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(vec![Event::key_release(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release))],
    );

    handler.assert_base_state();
}
