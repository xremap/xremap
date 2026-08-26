use crate::event::Event;
use crate::operator_handler::OperatorHandler;
use crate::tests::{assert_events, get_handler_from_config};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_millis(10);

fn get_dbltap_handler() -> OperatorHandler {
    get_handler_from_config(indoc! {"
        experimental_map:
          - remap:
              A:
                select:
                  - double: B
                    timeout: 5
                  - double: C
        "})
    .unwrap()
}

#[test]
fn test_expmap_two_double_taps_on_same_key_first_match() {
    let mut handler = get_dbltap_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_B)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_B)]);

    handler.assert_base_state();
}

#[test]
fn test_expmap_two_double_taps_on_same_key_last_match() {
    let mut handler = get_dbltap_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    std::thread::sleep(TIMEOUT); // Enough timeout, so first doesn't match.
    assert_events(handler.map_evs(vec![Event::Tick]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_C)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_C)]);

    handler.assert_base_state();
}

fn get_throttle_handler() -> OperatorHandler {
    get_handler_from_config(indoc! {"
        experimental_map:
          - remap:
              A:
                select:
                  - throttle_ms: 5
                  - double: B
        "})
    .unwrap()
}

#[test]
fn test_expmap_throttle_output_is_not_used_by_lower_priority_operator() {
    let mut handler = get_throttle_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_expmap_throttle_output_is_not_used_by_lower_priority_operator_double_press() {
    let mut handler = get_throttle_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);
    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);
    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

fn get_oneshot_handler() -> OperatorHandler {
    get_handler_from_config(indoc! {"
        experimental_map:
          - remap:
              A:
                select:
                  - double: B
                    timeout: 5
                  - oneshot: S_L
        "})
    .unwrap()
}

#[test]
fn test_expmap_oneshot_after_fast_double_tap() {
    let mut handler = get_oneshot_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_B)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_B)]);

    handler.assert_base_state();
}

#[test]
fn test_expmap_oneshot_after_slow_double_tap() {
    let mut handler = get_oneshot_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    std::thread::sleep(TIMEOUT);
    // dbltap cancels, and oneshot is emitted.
    assert_events(handler.map_evs(vec![Event::Tick]), vec![Event::key_press(Key::KEY_LEFTSHIFT)]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_K)]),
        vec![Event::key_press(Key::KEY_K), Event::key_release(Key::KEY_LEFTSHIFT)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}
