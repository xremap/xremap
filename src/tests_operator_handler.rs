use crate::config::expmap_operator::{DoubleTap, ExpmapAction, ExpmapOperator};
use crate::config::expmap_simkey::Simkey;
use crate::config::Expmap;
use crate::event::Event;
use crate::operator_handler::OperatorHandler;
use crate::operators::get_operator_handler;
use crate::tests::assert_events;
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_millis(10);

fn get_handler() -> OperatorHandler {
    let config: Vec<Expmap> = vec![
        Expmap {
            name: "".into(),
            chords: vec![],
            // Operators that interact on KEY_H
            // This has highest precedence.
            remap: HashMap::from([(
                Key::KEY_H,
                ExpmapOperator::DoubleTap(DoubleTap {
                    actions: vec![ExpmapAction::Key(Key::KEY_3), ExpmapAction::Key(Key::KEY_4)],
                    timeout: TIMEOUT,
                }),
            )]),
        },
        Expmap {
            name: "".into(),
            chords: vec![
                // Two operators that interact on KEY_B
                Simkey {
                    keys: vec![Key::KEY_A, Key::KEY_B],
                    actions: vec![ExpmapAction::Key(Key::KEY_1)],
                    timeout: TIMEOUT,
                },
                Simkey {
                    keys: vec![Key::KEY_B, Key::KEY_C],
                    actions: vec![ExpmapAction::Key(Key::KEY_2)],
                    timeout: TIMEOUT,
                },
                // Operators that interact on KEY_H
                Simkey {
                    keys: vec![Key::KEY_H, Key::KEY_I],
                    actions: vec![ExpmapAction::Key(Key::KEY_5)],
                    timeout: TIMEOUT,
                },
            ],
            remap: HashMap::new(),
        },
    ];

    get_operator_handler(&config, Rc::new(TimeoutManager::new())).unwrap()
}

#[test]
fn test_operator_handler_picks_heighest_candidate() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    // Highest matches
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![Event::key_press(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_heighest_candidate_after_lowest_has_canceled() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_H)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_H)]), vec![]);
    // Other key to let lowest remain canceled a while.
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_H)]),
        vec![
            Event::key_press(Key::KEY_3),
            Event::key_press(Key::KEY_4),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
        ],
    );
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_H)]),
        vec![Event::key_release(Key::KEY_3), Event::key_release(Key::KEY_4)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_lowest_candidate_when_heighest_cancels() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    // Lowest precedence match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    // Highest precedence cancels
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_2), Event::key_release(Key::KEY_2)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_C)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_finished_lowest_candidate_when_heighest_cancels() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    // Lowest precedence match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_C)]), vec![]);
    // Extra event
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![]);
    // Highest precedence cancels (and the lowest goes into done)
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_B)]),
        vec![
            Event::key_press(Key::KEY_2),
            Event::key_release(Key::KEY_2),
            Event::key_press(Key::KEY_K),
        ],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![Event::key_release(Key::KEY_K)]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_lowest_candidate_done_a_while_when_heighest_matches() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_H)]), vec![]);
    // Lowest match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_I)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_I)]), vec![]);
    // Lowest done
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_H)]), vec![]);

    // Other key to let lowest remain done a while.
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![]);

    // Highest precedence matches
    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_H)]),
        vec![
            Event::key_press(Key::KEY_3),
            Event::key_press(Key::KEY_4),
            Event::key_press(Key::KEY_I),
            Event::key_release(Key::KEY_I),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
        ],
    );

    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_H)]),
        vec![Event::key_release(Key::KEY_3), Event::key_release(Key::KEY_4)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_lowest_candidate_finished_a_while_when_heighest_cancels_by_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_H)]), vec![]);
    // Lowest match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_I)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_I)]), vec![]);
    // Lowest done
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_H)]), vec![]);

    // Other key to let lowest remain done a while.
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_K)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_K)]), vec![]);

    // Highest precedence cancels
    thread::sleep(TIMEOUT);
    assert_events(
        handler.map_evs(vec![Event::Tick]),
        vec![
            Event::key_press(Key::KEY_5),
            Event::key_release(Key::KEY_5),
            Event::key_press(Key::KEY_K),
            Event::key_release(Key::KEY_K),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_picks_lowest_candidate_when_canceled_by_timeout() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    // Lowest precedence match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    // Lowest precedence virtually emits
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_C)]), vec![]);

    // Highest precedence cancels, and lowest gets emitted.
    thread::sleep(TIMEOUT);
    assert_events(
        handler.map_evs(vec![Event::Tick]),
        vec![Event::key_press(Key::KEY_2), Event::key_release(Key::KEY_2)],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_two_candidates_that_both_cancel() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![]);
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_B), Event::key_release(Key::KEY_B)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_unhandled_events_are_passed_to_next_operator_when_canceled() {
    let mut handler = get_handler();

    // 1st operator
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    // 2nd operator (not candidate yet)
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    // Cancel 1st
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_press(Key::KEY_A)]);
    // Cancel 2nd
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_C)]),
        vec![
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_C),
        ],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_unhandled_events_are_passed_to_next_operator_at_match() {
    let mut handler = get_handler();

    // 1st operator
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    // 2nd operator (not candidate yet)
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_C)]), vec![]);
    // 1st match
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_B)]), vec![Event::key_press(Key::KEY_1)]);

    // 2nd cancel (can't match because 1st consumed KEY_B)
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_C)]),
        vec![Event::key_press(Key::KEY_C), Event::key_release(Key::KEY_C)],
    );
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_unhandled_events_are_passed_to_static_operators_when_done() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_press(Key::KEY_B)]),
        vec![Event::key_press(Key::KEY_1)],
    );

    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_A)]), vec![Event::key_release(Key::KEY_1)]);

    // Start new operator (of same type)
    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_A)]), vec![]);
    // Last event to 1st operator
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_B)]), vec![]);
    // Cancel 2nd operator
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_candidate_match_in_one_batch() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![
            Event::key_press(Key::KEY_A),
            Event::key_press(Key::KEY_B),
            Event::key_release(Key::KEY_A),
            Event::key_release(Key::KEY_B),
        ]),
        vec![Event::key_press(Key::KEY_1), Event::key_release(Key::KEY_1)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_candidate_cancels_in_one_batch() {
    let mut handler = get_handler();

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)]),
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}

#[test]
fn test_operator_handler_candidate_with_many_actions() {
    let mut handler = get_handler();

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_H)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_H)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_O)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_O)]), vec![]);

    assert_events(handler.map_evs(vec![Event::key_press(Key::KEY_P)]), vec![]);
    assert_events(handler.map_evs(vec![Event::key_release(Key::KEY_P)]), vec![]);

    assert_events(
        handler.map_evs(vec![Event::key_press(Key::KEY_H)]),
        vec![
            Event::key_press(Key::KEY_3),
            Event::key_press(Key::KEY_4),
            Event::key_press(Key::KEY_O),
            Event::key_release(Key::KEY_O),
            Event::key_press(Key::KEY_P),
            Event::key_release(Key::KEY_P),
        ],
    );
    assert_events(
        handler.map_evs(vec![Event::key_release(Key::KEY_H)]),
        vec![Event::key_release(Key::KEY_3), Event::key_release(Key::KEY_4)],
    );

    handler.assert_base_state();
    handler.assert_emitted_modifiers_are_synced();
}
