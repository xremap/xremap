use crate::config::expmap_operator::ExpmapOperator;
use crate::config::Expmap;
use crate::emit_handler::{Emit, EmitHandler};
use crate::event::Event;
use crate::event_handler::PRESS;
use crate::operator_double_tap::DoubleTapOperator;
use crate::operator_sim::SimOperator;
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use std::collections::HashMap;
use std::rc::Rc;
use std::usize;

pub struct OperatorHandler {
    active: Vec<Box<dyn ActiveOperator>>,
    candidates: Option<Candidates>,
    lookup_map: HashMap<Key, Vec<Box<dyn StaticOperator>>>,
    emit_handler: EmitHandler,
}

/// Operators start matching when their start_key is pressed.
/// Operators remain candidates until there is only one candidate left,
/// and that operator then becomes active, and is placed as the right-most operator.
/// Operators are given events from the left first, and can choose to emit to the next
/// operator or to the standard modmap.
/// The logic is:
///   1. Try all active operators in the order they ware activated.
///   2. Try all candidates. They have no order, and there's no active operators after
///      the candidates.
///   3. Lookup operators that have the keypress as start_key.
///   4. Let the event go to next level.
///
/// Candidates are needed to handle two operators like: Simultaneous(A,B) and Simultaneous(A,C). It's
/// not possible to determine which mapping to choose, when A is pressed. So the events
/// must be buffered until it's known which operator should handle the event.
///
/// Active operators are needed to handle Simultaneous(A,B) -> 1. The events 'ABaAbB' has an overlap
/// where the first operator handles the first part 'ABaAb' -> (1,A), and the second operator
/// handles the last part 'AB'. If operators were only static, then it would be more complicated
/// because they would have to keep track of the whether 'b' should be squashed or let through.
impl OperatorHandler {
    pub fn new(experimental_map: &Vec<Expmap>, timeout_manager: Rc<TimeoutManager>) -> OperatorHandler {
        let mut lookup_map: HashMap<Key, Vec<Box<dyn StaticOperator>>> = HashMap::new();

        for expmap in experimental_map {
            for chord in &expmap.chords {
                let operator = Box::new(SimOperator {
                    keys: chord.keys.clone(),
                    actions: chord.actions.clone(),
                    timeout: chord.timeout,
                    timeout_manager: timeout_manager.clone(),
                });
                for (key, op) in operator.get_operators() {
                    match lookup_map.get_mut(&key) {
                        Some(current) => {
                            current.push(op);
                        }
                        None => {
                            lookup_map.insert(key, vec![op]);
                        }
                    };
                }
            }

            for (key, op) in &expmap.remap {
                let operator = match op {
                    ExpmapOperator::DoubleTap(dbltap) => Box::new(DoubleTapOperator {
                        key: key.clone(),
                        actions: dbltap.actions.clone(),
                        timeout: dbltap.timeout,
                        timeout_manager: timeout_manager.clone(),
                    }),
                };
                for (key, op) in operator.get_operators() {
                    match lookup_map.get_mut(&key) {
                        Some(current) => {
                            current.push(op);
                        }
                        None => {
                            lookup_map.insert(key, vec![op]);
                        }
                    };
                }
            }
        }

        OperatorHandler {
            active: vec![],
            candidates: None,
            lookup_map,
            emit_handler: EmitHandler::new(),
        }
    }

    #[cfg(test)]
    pub fn assert_base_state(&self) {
        assert!(self.active.is_empty());
        assert!(self.candidates.is_none());
    }

    #[cfg(test)]
    pub fn assert_emitted_modifiers_are_synced(&self) {
        self.emit_handler.assert_emitted_modifiers_are_synced();
    }

    #[cfg(test)]
    pub fn map_evs(&mut self, events: Vec<Event>) -> Vec<Event> {
        self.map_events(events)
    }

    pub fn map_events(&mut self, events: Vec<Event>) -> Vec<Event> {
        events
            .into_iter()
            .flat_map(|event| {
                self.emit_handler.on_event(&event);

                let events = process_event(event, &mut self.active, &mut self.candidates, &self.lookup_map);

                self.emit_handler.map_output(events)
            })
            .collect()
    }
}

#[derive(Debug)]
enum CandidateState {
    Canceled,
    Done,
    Matching,
}

#[derive(Debug)]
struct Candidate {
    operator: Box<dyn ActiveOperator>,
    state: CandidateState,
    emitted: Vec<Emit>,
    unhandled: Vec<Event>,
}

#[derive(Debug)]
struct Candidates {
    // start_key and events are given to the candidates, and buffered here
    // in case all candidates cancel.
    start_event: Event,
    events: Vec<Event>,
    operators: Vec<Candidate>,
}

// Nodes that exist on the stack, that still needs to be processed.
enum Node {
    Event(Event),
    Operator(Box<dyn ActiveOperator>),
    CandidateChosen(usize),
    CandidatesCanceled,
}

// The point that is being processed is in the middle of the left-stack and the right-operators.
// Candidates only exist if there is no right-operators.
// The right-most event is always processed first. This is also true for the events emitted during
// processing.
fn process_event(
    event: Event,
    right: &mut Vec<Box<dyn ActiveOperator>>,
    candidates: &mut Option<Candidates>,
    lookup_map: &HashMap<Key, Vec<Box<dyn StaticOperator>>>,
) -> Vec<Emit> {
    // The events that have passed fully through the operators.
    let mut emit: Vec<Emit> = vec![];
    // The stack, that still needs processing.
    let mut left: Vec<Node> = vec![Node::Event(event)];

    loop {
        match left.pop() {
            Some(Node::Event(event)) => {
                match right.pop() {
                    // Map the event with the operator to the right.
                    Some(mut operator) => match operator.on_event(&event) {
                        OperatorAction::Undecided => {
                            right.push(operator);
                        }
                        OperatorAction::Cancel => {
                            unreachable!()
                        }
                        OperatorAction::Unhandled => {
                            // Move operator one to the left
                            left.push(Node::Operator(operator));
                            left.push(Node::Event(event));
                        }
                        OperatorAction::Partial(emitted, unhandled) => {
                            // Leave operator where it is.
                            right.push(operator);
                            emit.extend(emitted);

                            unhandled_back_to_stack(unhandled, &mut left);
                        }
                        OperatorAction::Done(new_emit, unhandled) => {
                            // Implicitly drops operator
                            emit.extend(new_emit);
                            unhandled_back_to_stack(unhandled, &mut left);
                        }
                    },

                    // No more operators
                    None => {
                        match candidates {
                            Some(candidates) => try_candidates(event, &mut left, candidates),
                            None => static_lookup(event, candidates, &lookup_map, &mut emit),
                        };
                    }
                };
            }

            Some(Node::Operator(operator)) => {
                right.push(operator);
            }

            Some(Node::CandidateChosen(chosen)) => {
                let candidate = candidates.take().unwrap().operators.into_iter().nth(chosen).unwrap();

                emit.extend(candidate.emitted);

                if !matches!(candidate.state, CandidateState::Done) {
                    right.push(candidate.operator);
                }

                unhandled_back_to_stack(candidate.unhandled, &mut left);
            }

            Some(Node::CandidatesCanceled) => {
                let taken = candidates.take().unwrap();

                // start_key didn't match anything, so emit.
                emit.push(Emit::Single(taken.start_event.clone()));
                // Start over with the next event.
                unhandled_back_to_stack(taken.events, &mut left);
            }

            None => {
                // No more nodes to process, so all events have gone through the pipeline.
                return emit;
            }
        };
    }
}

fn unhandled_back_to_stack(events: Vec<Event>, nodes: &mut Vec<Node>) {
    nodes.extend(
        events
            .iter()
            .rev()
            .map(|event| Node::Event(event.clone()))
            .collect::<Vec<_>>(),
    );
}

fn try_candidates(event: Event, left: &mut Vec<Node>, candidates: &mut Candidates) {
    candidates.events.push(event.clone());

    let mut first = true;

    for (usize, candidate) in candidates.operators.iter_mut().enumerate() {
        match candidate.state {
            CandidateState::Canceled => {
                continue;
            }
            CandidateState::Done => {
                if first {
                    if !matches!(event, Event::Tick) {
                        todo!()
                    }
                    candidate.unhandled.push(event.clone());

                    left.push(Node::CandidateChosen(usize));
                    return;
                } else {
                    candidate.unhandled.push(event.clone());
                }
            }
            CandidateState::Matching => {
                match candidate.operator.on_event(&event) {
                    OperatorAction::Undecided => {
                        first = false;
                    }
                    OperatorAction::Cancel => {
                        candidate.state = CandidateState::Canceled;
                    }
                    OperatorAction::Unhandled => {
                        candidate.unhandled.push(event.clone());

                        if first {
                            left.push(Node::CandidateChosen(usize));
                            return;
                        }
                    }
                    OperatorAction::Partial(new_emit, unhandled) => {
                        candidate.emitted.extend(new_emit);
                        candidate.unhandled.extend(unhandled);

                        if first {
                            left.push(Node::CandidateChosen(usize));
                            return;
                        }
                    }
                    OperatorAction::Done(new_emit, unhandled) => {
                        candidate.emitted.extend(new_emit);
                        candidate.unhandled.extend(unhandled);
                        candidate.state = CandidateState::Done;

                        if first {
                            left.push(Node::CandidateChosen(usize));
                            return;
                        }
                    }
                };
            }
        };
    }

    if first {
        // all canceled
        left.push(Node::CandidatesCanceled);
    }
}

fn static_lookup(
    event: Event,
    candidates: &mut Option<Candidates>,
    lookup_map: &HashMap<Key, Vec<Box<dyn StaticOperator>>>,
    emit: &mut Vec<Emit>,
) {
    let (device, key_event) = match &event {
        Event::KeyEvent(device, key_event) => (device, key_event),
        Event::Tick => {
            // Ignore, because static operators can't be triggered by timeout.
            return;
        }
        _ => {
            emit.push(Emit::Single(event));
            return;
        }
    };

    if key_event.value() != PRESS {
        emit.push(Emit::key_event(device.clone(), key_event.clone()));
        return;
    }

    // Static operators
    match lookup_map.get(&key_event.key) {
        Some(operators) => {
            debug_assert!(!operators.is_empty());
            debug_assert!(candidates.is_none());

            let new_candidates: Vec<_> = operators
                .iter()
                .map(|operator| Candidate {
                    operator: operator.get_active_operator(&event),
                    state: CandidateState::Matching,
                    emitted: vec![],
                    unhandled: vec![],
                })
                .collect();

            candidates.replace(Candidates {
                start_event: event,
                events: vec![],
                operators: new_candidates,
            });
        }
        None => {
            emit.push(Emit::key_event(device.clone(), key_event.clone()));
        }
    };
}
