use crate::config::tests::assert_invalid_config;
use crate::event::Event;
use crate::tests::{assert_events, get_handler_from_config};
use evdev::KeyCode as Key;
use indoc::indoc;

#[test]
fn test_expmap_actions_with_null_actions() {
    let mut handler = get_handler_from_config(indoc! {"
        experimental_map:
            - chords:
                - keys: [A, B]
                  actions: null
        "})
    .unwrap();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A), Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
}

#[test]
fn test_expmap_actions_with_empty_actions() {
    let mut handler = get_handler_from_config(indoc! {"
        experimental_map:
            - chords:
                - keys: [A, B]
                  actions: []
        "})
    .unwrap();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A), Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
}

#[test]
fn test_expmap_actions_with_int_actions() {
    assert_invalid_config(
        indoc! {"
        experimental_map:
            - chords:
                - keys: [A, B]
                  actions: 1
    "},
        "experimental_map[0].chords[0]: data did not match any variant of untagged enum ParseExpmapActions at line 3 column 11",
    );
}

#[test]
fn test_expmap_actions_with_multiple_actions() {
    let mut handler = get_handler_from_config(indoc! {"
        experimental_map:
            - chords:
                - keys: [A, B]
                  actions: [C, '1']
        "})
    .unwrap();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_C), Event::key_press(Key::KEY_1)],
    );
    assert_events(
        handler.map_evs(vec![Event::key_repeat(Key::KEY_A), Event::key_repeat(Key::KEY_B)]),
        vec![Event::key_repeat(Key::KEY_C), Event::key_repeat(Key::KEY_1)],
    );
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A), Event::key_release(Key::KEY_B)]),
        vec![Event::key_release(Key::KEY_C), Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
}
