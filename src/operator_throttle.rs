use crate::config::expmap_actions::ExpmapActions;
use crate::config::expmap_operator::ThrottleDefinition;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent};
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use crate::KeyValue;
use evdev::KeyCode as Key;
use std::rc::Rc;
use std::time::Instant;
use std::vec;

#[derive(Debug, Clone)]
pub struct ThrottleOperator {
    key: Key,
    definition: ThrottleDefinition,
}

impl ThrottleOperator {
    pub fn get_ops(key: Key, definition: ThrottleDefinition) -> Vec<(Key, Box<dyn StaticOperator>)> {
        vec![(key, Box::new(ThrottleOperator { key, definition }))]
    }
}

impl StaticOperator for ThrottleOperator {
    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator> {
        match event {
            Event::KeyEvent(_, _) => Box::new(ActiveThrottleOperator {
                key: self.key,
                definition: self.definition.clone(),
                actions: ExpmapActions { keys: vec![self.key] },
                last_emit: Instant::now(),
                state: State::New,
            }),
            _ => {
                unreachable!()
            }
        }
    }
}

#[derive(Debug)]
enum State {
    New,
    // Press is emitted, waiting for repeat/release.
    Active,
    // Press is squashed, waiting for repeat/release.
    Squash,
    // Key not physically pressed, waiting for timeout.
    Inactive,
    Done,
}

#[derive(Debug)]
struct ActiveThrottleOperator {
    key: Key,
    definition: ThrottleDefinition,
    actions: ExpmapActions,
    // Is set to 'now' just before entering Active state with press-event.
    last_emit: Instant,
    state: State,
}

impl ActiveOperator for ActiveThrottleOperator {
    fn on_press(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::New => {
                debug_assert_eq!(key_event.key, self.key);

                self.state = State::Active;
                self.last_emit = Instant::now();
                self.map_actions(device, KeyValue::Press)
            }
            State::Active => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if Instant::now().duration_since(self.last_emit) > self.definition.timeout {
                    if key_event.key == self.key {
                        // Pressed again before timeout.
                        self.state = State::New;
                        self.on_press(device, key_event)
                    } else {
                        self.state = State::Done;
                        OperatorAction::Done(vec![Event::key_event2(device, key_event.clone())])
                    }
                } else {
                    if key_event.key == self.key {
                        // Begin squash
                        self.state = State::Squash;
                        OperatorAction::Partial(vec![])
                    } else {
                        OperatorAction::Unhandled
                    }
                }
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_release(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::New => unreachable!(),
            State::Active => {
                if key_event.key == self.key {
                    self.state = State::Inactive;
                    self.map_actions(device, KeyValue::Release)
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    self.state = State::Inactive;
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_repeat(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::New => unreachable!(),
            State::Active => {
                if key_event.key == self.key {
                    self.map_actions(device, KeyValue::Repeat)
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_tick(&mut self) -> OperatorAction {
        OperatorAction::Unhandled
    }

    fn on_other(&mut self, _event: &Event) -> OperatorAction {
        OperatorAction::Unhandled
    }
}

impl ActiveThrottleOperator {
    fn map_actions(&self, device: Rc<InputDeviceInfo>, value: KeyValue) -> OperatorAction {
        OperatorAction::Partial(self.actions.emit(device, value))
    }
}
