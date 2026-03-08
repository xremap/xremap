use crate::config::expmap_operator::ExpmapAction;
use crate::event::Event;
use crate::operator_double_tap::DoubleTapOperator;
use crate::operator_handler::OperatorHandler;
use crate::operators::StaticOperator;
use crate::tests::assert_events;
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_millis(10);

fn get_handler() -> OperatorHandler {
    let timeout_manager = Rc::new(TimeoutManager::new());

    let mut operators: Vec<Box<dyn StaticOperator>> = vec![];

    operators.push(Box::new(DoubleTapOperator {
        key: Key::KEY_LEFTCTRL,
        actions: vec![ExpmapAction::Key(Key::KEY_1)],
        timeout: TIMEOUT,
        timeout_manager: timeout_manager.clone(),
    }));

    OperatorHandler::new(operators)
}

#[test]
fn test_dbltap_key_not_matching() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_at_first_press_not_canceled_by_other_non_matching() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTALT)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_press(Key::KEY_LEFTALT)],
    );

    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
}

#[test]
fn test_dbltap_at_first_press_canceled_by_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(
        handler.map_events(vec![Event::Tick]),
        vec![Event::key_press(Key::KEY_LEFTCTRL), Event::key_press(Key::KEY_A)],
    );
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_LEFTCTRL)],
    );

    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_A)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_at_first_press_repeated() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_repeat(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_when_tapped_not_canceled_by_other_non_matching() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTALT)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_press(Key::KEY_LEFTALT)],
    );
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTALT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_when_tapped_canceled_by_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(
        handler.map_events(vec![Event::Tick]),
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_when_tapped_canceled_by_timeout_with_rolled_key() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_A)]), vec![]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(
        handler.map_events(vec![Event::Tick]),
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_release(Key::KEY_A),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_when_tapped_canceled_by_timeout_with_modded_key() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_A)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(
        handler.map_events(vec![Event::Tick]),
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_LEFTCTRL),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_when_tapped_canceled_by_timeout_with_distinct_key() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_A)]), vec![]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(
        handler.map_events(vec![Event::Tick]),
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_release(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_spurious_events() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    // Spurious release
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    // Spurious repeat
    assert_events(handler.map_events(vec![Event::key_repeat(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);

    // Spurious press
    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_twice() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_then_tick_before_release() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);

    thread::sleep(TIMEOUT);

    assert_events(handler.map_events(vec![Event::Tick]), vec![]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_then_repeat_triggering_key() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(handler.map_events(vec![Event::key_repeat(Key::KEY_LEFTCTRL)]), vec![Event::key_repeat(Key::KEY_1)]);
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_then_repeat_ordinary_key() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_A)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_repeat(Key::KEY_A)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_press(Key::KEY_A)],
    );

    assert_events(handler.map_events(vec![Event::key_repeat(Key::KEY_A)]), vec![Event::key_repeat(Key::KEY_A)]);

    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_surrounded_at_first_press() {
    let mut handler = get_handler();

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTALT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTALT)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_surrounded_at_tapped() {
    let mut handler = get_handler();

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTALT)]),
        vec![Event::key_press(Key::KEY_LEFTALT)],
    );

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTALT)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_release(Key::KEY_LEFTALT)],
    );
    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_dbltap_surrounded_at_double_tapped() {
    let mut handler = get_handler();

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]), vec![]);
    assert_events(handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]), vec![]);

    assert_events(handler.map_events(vec![Event::key_press(Key::KEY_LEFTALT)]), vec![]);

    assert_events(
        handler.map_events(vec![Event::key_press(Key::KEY_LEFTCTRL)]),
        vec![Event::key_press(Key::KEY_1), Event::key_press(Key::KEY_LEFTALT)],
    );

    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTALT)]),
        vec![Event::key_release(Key::KEY_LEFTALT)],
    );

    assert_events(
        handler.map_events(vec![Event::key_release(Key::KEY_LEFTCTRL)]),
        vec![Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}
