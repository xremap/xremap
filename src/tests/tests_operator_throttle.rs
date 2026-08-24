use crate::event::Event;
use crate::operator_handler::OperatorHandler;
use crate::tests::{assert_events, get_handler_from_config};
use evdev::KeyCode as Key;
use indoc::indoc;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_millis(10);

fn get_handler() -> OperatorHandler {
    get_handler_from_config(indoc! {"
        experimental_map:
            - remap:
                A: { throttle_ms: 10 }
        "})
    .unwrap()
}

#[test]
fn test_throttle_mess() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_squash() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    // Events are squashed
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_timeout_at_key_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT); // The key event is the first after timeout.
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT); // So it goes into done state
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_timeout_at_other_key_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_surrounded_at_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_surrounded_at_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_surrounded_at_press_inactive() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_surrounded_at_release_inactive() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}

#[test]
fn test_throttle_spurious() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]); // spurious
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![]); // spurious

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]); // spurious
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]); // spurious

    std::thread::sleep(TIMEOUT);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
}
