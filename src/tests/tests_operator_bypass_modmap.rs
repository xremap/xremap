use crate::action::Action;
use crate::event::{Event, KeyEvent};
use crate::tests::EventHandlerForTest;
use crate::KeyValue;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_modmap_bypass() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - bypass: true
              remap:
                A: B
        keymap:
            - remap:
                A: K
                B: K
        "});

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
fn test_modmap_bypass_mul_purpose_key() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - bypass: true
              remap:
                A:
                    held: B
                    alone: C
        keymap:
            - remap:
                A: K
                B: K
                C: K
                D: { press: E }
        "});

    // Tap
    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );

    // Hold - Interrupting key is not affected by bypass=true
    handler.assert(vec![Event::key_press(Key::KEY_A)], vec![]);
    handler.assert(
        vec![Event::key_press(Key::KEY_D)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_E, KeyValue::Press)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_D)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release))],
    );
    handler.assert(
        vec![Event::key_repeat(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Repeat))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_modmap_bypass_press_release_key_is_bypass_by_default() {
    // press, release and repeat are bypass by default.
    // Original key is on the other hand not bypass by default.
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - remap:
                A:
                  press: { press: B }
                  release: { release: B }
        keymap:
            - remap:
                A: { press: '1' }
                B: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );

    handler.assert_base_state();
}

#[test]
fn test_modmap_bypass_press_release_key() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - bypass: true
              remap:
                A:
                  press: B
        keymap:
            - remap:
                A: '1'
                B: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_modmap_bypass_then_match_key_combo() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        modmap:
            - bypass: true
              remap:
                capslock: S_L
        keymap:
            - remap:
                S-A: C-B
                S_L: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_CAPSLOCK)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_CAPSLOCK)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_modmap_bypass_virtual_modifier_is_emitted() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        virtual_modifiers: [capslock]
        modmap:
            - bypass: true
              remap:
                f12: capslock
        keymap:
            - remap:
                f12: K
                capslock: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_F12)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_F12)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_modmap_bypass_then_match_key_combo_with_virtual_modifier() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        virtual_modifiers: [capslock]
        modmap:
            - bypass: true
              remap:
                f12: capslock
        keymap:
            - remap:
                capslock-A: C-B
                capslock: K
        "});

    handler.assert(
        vec![Event::key_press(Key::KEY_F12)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Press))],
    );
    handler.assert(
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release))],
    );
    handler.assert(
        vec![Event::key_release(Key::KEY_F12)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release))],
    );

    handler.assert_base_state();
}

#[test]
fn test_global_bypass_used_in_modmap() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        bypass: true
        modmap:
            - remap:
                A: B
        keymap:
            - remap:
                A: K
                B: K
        "});

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
fn test_global_bypass_overwritten_in_modmap() {
    let mut handler = EventHandlerForTest::new(indoc! {"
        bypass: true
        modmap:
            - bypass: false
              remap:
                A: B
        keymap:
            - remap:
                A: C
                B: D
        "});

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
