use crate::event::Event;
use crate::operator_handler::OperatorHandler;
use crate::tests::{assert_events, get_handler_from_config};
use evdev::KeyCode as Key;
use indoc::indoc;

fn get_handler() -> OperatorHandler {
    get_handler_from_config(indoc! {"
        experimental_map:
            - remap:
                s_l: { oneshot: a_l }
        "})
    .unwrap()
}

#[test]
fn test_oneshot_used() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(
        handler.map_evs(vec![Event::key_repeat(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_repeat(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    // spurious
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_cancelled_by_repress() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
        ],
    );
    assert_events(
        handler.map_evs(vec![Event::key_repeat(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_repeat(Key::KEY_LEFTSHIFT)],
    );
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTSHIFT)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_interrupted_then_released_staggered() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_A)]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_interrupted_by_two_keys() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_B)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_B)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_interrupted_then_released_modded() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    // spurious
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_used_with_two_keys_in_batch() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_K)]),
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_K),
        ],
    );

    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A), Event::key_release(Key::KEY_K)]),
        vec![Event::key_release(Key::KEY_A), Event::key_release(Key::KEY_K)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_used_with_press_and_repeat_in_batch() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_repeat(Key::KEY_A)]),
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_repeat(Key::KEY_A),
        ],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_surround_at_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_K)]), vec![Event::key_repeat(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_surround_press_and_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_K)]), vec![Event::key_repeat(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_surround_cancelling_release() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
        ],
    );

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTSHIFT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_oneshot_surround_cancelling_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
        ],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTSHIFT)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_spuriously_pressed() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    // Spurious in pressed-state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);
    // Spurious in oneshot-state
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]), vec![]);

    // Tap again to cancel oneshot state.
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![
            Event::key_release(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
        ],
    );
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTSHIFT)],
    );

    handler.assert_base_state();
}

#[test]
fn test_oneshot_spuriously_when_interrupted() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);

    //spurious
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_LEFTSHIFT)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_LEFTSHIFT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );

    handler.assert_base_state();
}
