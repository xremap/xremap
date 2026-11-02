use crate::action::Action;
use crate::config::tests::assert_invalid_config;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

#[test]
fn test_exact_match_true_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_H),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_exact_match_false_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: false
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_H),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_nested_remap_with_null_action() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    A: null
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_A)],
        vec![],
    );
}

#[test]
fn test_nested_remap_with_empty_action() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    A: []
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_A)],
        vec![],
    );
}

#[test]
fn test_nested_remap_with_int_action() {
    assert_invalid_config(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    A: 1
    "},
        "keymap[0].remap: data did not match any variant of untagged enum VectorOrSingleOrNull at line 3 column 7",
    );
}

#[test]
fn test_nested_remap_with_multiple_actions() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    A: [C, A-D, '1']
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}

#[test]
fn test_nested_remap_does_not_match_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    C-A: B
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_A)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
    );
}

#[test]
fn test_nested_remap_does_not_match_mod_trigger() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                    C-A_L: B
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_press(Key::KEY_LEFTALT)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))],
    );
}

#[test]
fn test_merge_remaps() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  k: C-w
    "};

    // Match in first
    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_H),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    // Match in second
    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_K),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_merge_remaps_precedence_to_first() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  h: C-b
                  c: C-q
    "};

    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_H),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_release(Key::KEY_X),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_non_remap_actions_are_ignored_in_merge() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              # Does not suddenly emit actions, when collecting nested remaps.
              C-x: K
          - remap:
              C-x:
                remap:
                  # Is prioritized as exact match, even though first can match inexact.
                  C-h: C-b
    "};

    assert_actions(
        config,
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_press(Key::KEY_H),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    );
}

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

    // Match first
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

    // Match second
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
fn test_merge_nested_sibling_remaps_timeout_key_precedence_to_first() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              CAPSLOCK:
                - timeout_millis: 100
                  timeout_key: left
                  remap:
                    a: b
                - timeout_millis: 100
                  timeout_key: right
                  remap:
                    c: d
    "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::OverrideTimeout,
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFT, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_merge_nested_sibling_remaps_only_timeout_info_from_first_sibling_is_used() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              CAPSLOCK:
                - remap:
                    a: b
                - timeout_millis: 100
                  timeout_key: right
                  remap:
                    c: d
    "},
        vec![
            Event::key_press(Key::KEY_CAPSLOCK),
            Event::key_release(Key::KEY_CAPSLOCK),
            Event::OverrideTimeout,
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_CAPSLOCK, KeyValue::Release))],
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
fn test_modifier_does_not_cancel_remap_and_is_emitted() {
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
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
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
fn test_cancel_by_timeout_emits_nothing_by_default() {
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
            // Confusing, that timeout_key isn't emitted here.
            // It happens because `remove_override` is called in `find_keymap`,
            //  if a key exists in the lookup-table, before modifiers are considered.
            //  Then on non-match it falls through to `timeout_override`, which has nothing
            //  to timeout, because it was 'maybe-wrongly' cancelled.
            // Or maybe it's confusing, when timeout_key is emitted on cancel. Both possible.
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_keypress_and_remap_in_keymap_action() {
    // KEY_D will be emitted, and the remap will be used for next key press.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                - d
                - remap:
                    a: b
        "},
        vec![
            Event::key_press(Key::KEY_F12),
            Event::key_release(Key::KEY_F12),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_no_keypress_and_remap_in_keymap_action() {
    // The first match stops the search for matches. So the last remap isn't used.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
          - remap:
              f12:
                - remap:
                    a: b
        "},
        vec![
            Event::key_press(Key::KEY_F12),
            Event::key_release(Key::KEY_F12),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_nested_remap_inexact_when_union_of_modifiers() {
    // First is used when both match
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                c-x:
                  remap:
                    alt-W: A
                    control-W: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_X),
            Event::key_press(Key::KEY_W),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_triple_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              A:
                remap:
                  B:
                    remap:
                      C: D
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_B),
            Event::key_press(Key::KEY_C),
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
fn test_timeout_emits_lastest_nested_remap_trigger() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                  A:
                    timeout_millis: 100
                    remap:
                      B: C
        "},
        vec![
            Event::key_press(Key::KEY_F12),
            Event::key_press(Key::KEY_A),
            Event::OverrideTimeout,
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_timeout_is_replaced_at_each_remap_level() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                timeout_millis: 100
                remap:
                  A:
                    # no-timeout is respected here
                    remap:
                      B: C
        "},
        vec![
            Event::key_press(Key::KEY_F12),
            Event::key_press(Key::KEY_A),
            // Even though a cancel at level-A would have emitted the timeout key.
            // Cancel here at level-B doesn't care about the timeout setting in the previous level.
            Event::key_press(Key::KEY_K),
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press))],
    );
}

#[test]
fn test_timeout_key_is_emitted_as_taps() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              C-f12:
                timeout_millis: 100
                timeout_key: [X, '1']
                remap:
                  A: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_F12),
            Event::key_press(Key::KEY_K),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_1, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
    );
}

#[test]
fn test_timeout_key_with_null_actions() {
    assert_invalid_config(
        indoc! {"
        keymap:
          - remap:
              C-f12:
                timeout_millis: 100
                timeout_key: null
                remap:
                  A: B
    "},
        "keymap[0].remap: data did not match any variant of untagged enum VectorOrSingleOrNull at line 3 column 7",
    );
}

#[test]
fn test_timeout_key_with_empty_actions() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              c_l-f12:
                timeout_millis: 100
                timeout_key: []
                remap:
                  A: B
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_F12),
            Event::key_press(Key::KEY_K),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
    );
}

#[test]
fn test_timeout_key_with_int_actions() {
    assert_invalid_config(
        indoc! {"
        keymap:
          - remap:
              c_l-f12:
                timeout_millis: 100
                timeout_key: 1
                remap:
                  A: B
    "},
        "keymap[0].remap: data did not match any variant of untagged enum VectorOrSingleOrNull at line 3 column 7",
    );
}

#[test]
fn test_modifier_does_not_cancel_triple_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              A:
                remap:
                  B:
                    remap:
                      S-C: D
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_B),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_remap_modifier_in_triple_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              A:
                remap:
                  B:
                    remap:
                      s_l: C
                      D: E
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_B),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_D),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            // Modifiers don't cancel remap, even if modifiers trigger remapping.
            Action::KeyEvent(KeyEvent::new(Key::KEY_E, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_E, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_modifier_with_nested_remap_in_triple_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              A:
                remap:
                  s_l:
                    remap:
                      B: C
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_B),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_remap_does_not_match_modifiers_in_triple_nested_remap() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                remap:
                  A:
                    remap:
                      C-A_L: D
        "},
        vec![
            Event::key_press(Key::KEY_F12),
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_LEFTALT),
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))],
    );
}
