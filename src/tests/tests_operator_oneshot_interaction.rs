use crate::{event::Event, tests::*};
use evdev::KeyCode as Key;
use indoc::indoc;

#[test]
fn test_oneshot_used_by_dbltap_key() {
    let mut handler = get_handler_from_config(indoc! {"
        experimental_map:
            - remap:
                A: { double: B }
                s_l: { oneshot: s_l }
        "})
    .unwrap();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTSHIFT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_B), Event::key_release(Key::KEY_LEFTSHIFT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_B)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_interrupted_by_dbltap_key() {
    let mut handler = get_handler_from_config(indoc! {"
        experimental_map:
            - remap:
                A: { double: B }
                s_l: { oneshot: s_l }
        "})
    .unwrap();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTSHIFT)],
    );

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_B)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_B)]);

    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTSHIFT)],
    );

    handler.assert_base_state();
}
