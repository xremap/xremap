use crate::config::expmap_operator::ExpmapAction;
use crate::config::expmap_simkey::Simkey;
use crate::config::Expmap;
use crate::event::Event;
use crate::operator_handler::OperatorHandler;
use crate::tests::assert_events;
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_millis(10);

fn get_handler() -> OperatorHandler {
    let config: Vec<Expmap> = vec![Expmap {
        name: "".into(),
        chords: vec![
            Simkey {
                keys: vec![Key::KEY_A, Key::KEY_B],
                actions: vec![ExpmapAction::Key(Key::KEY_1)],
                timeout: TIMEOUT,
            },
            Simkey {
                keys: vec![Key::KEY_C, Key::KEY_D, Key::KEY_E],
                actions: vec![ExpmapAction::Key(Key::KEY_2)],
                timeout: TIMEOUT,
            },
        ],
        remap: HashMap::new(),
    }];

    OperatorHandler::new(&config, Rc::new(TimeoutManager::new()))
}

#[test]
fn symkey_test_first_key_released_before_second_is_pressed() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_pressed_then_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(handler.map_evs(vec![Event::Tick]), vec![Event::key_press(Key::KEY_A)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_pressed_then_release_of_old_other_key() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(handler.map_evs(vec![Event::Tick]), vec![Event::key_press(Key::KEY_B)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_B),
            Event::key_release(Key::KEY_A),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn simkey_test_releasing_key_does_not_start_matching() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_emitted_then_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);

    thread::sleep(TIMEOUT);

    assert_events(handler.map_evs(vec![Event::Tick]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_emitted_key_is_released_when_first_trigger_key_is_released() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_trigger_then_modded_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_1)]);
    // Modded release order
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_trigger_then_rolled_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_1)]);
    // Rolled release order
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_trigger_in_reverse_then_modded_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);
    // Modded release order
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_trigger_in_reverse_then_rolled_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);
    // Rolling release order
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_released_then_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);

    thread::sleep(TIMEOUT);

    assert_events(handler.map_evs(vec![Event::Tick]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_second_key_can_start_matching_right_after_other_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
    );

    // Matching starts on the other key.
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_B), Event::key_release(Key::KEY_B)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_surrounded_at_first_press() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_1), Event::key_release(Key::KEY_K)],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_surrounded_at_first_press_and_cancel() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_K),
            Event::key_release(Key::KEY_A),
        ],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_B)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_surrounded_at_emit() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_1), Event::key_press(Key::KEY_K)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_surrounded_at_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_surrounded_at_done() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_repeat() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![Event::key_press(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);

    // Only one of the trigger keys send repeat events.
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_B)]), vec![]);

    // Unrelated repeat
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_K)]), vec![Event::key_repeat(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);

    // Repeat must be squashed, because the action has been released.
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_B)]), vec![]);

    // Repeat unrelated key
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_K)]), vec![Event::key_repeat(Key::KEY_K)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_repeat_after_release() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);

    // Only one of the trigger keys send repeat events.
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_B)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);

    // Repeat must be squashed, because action has been released.
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_B)]), vec![]);

    // Repeat trigger key after release (it's buffered until canceled by A-release)
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_repeat(Key::KEY_A)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_repeat(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn symkey_test_3_simkeys() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_E)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_D)]), vec![Event::key_press(Key::KEY_2)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_D)]), vec![Event::key_release(Key::KEY_2)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_C)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_E)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}
