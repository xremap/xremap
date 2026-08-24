use crate::device::InputDeviceInfo;
use crate::emit_handler::Emit;
use crate::event::{Event, KeyEvent};
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use evdev::KeyCode as Key;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec;

#[derive(Debug, Clone)]
pub struct ThrottleOperator {
    key: Key,
    timeout: Duration,
}

impl ThrottleOperator {
    pub fn get_ops(key: Key, timeout: Duration) -> Vec<(Key, Box<dyn StaticOperator>)> {
        vec![(key, Box::new(ThrottleOperator { key, timeout }))]
    }
}

impl StaticOperator for ThrottleOperator {
    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator> {
        match event {
            Event::KeyEvent(_, _) => Box::new(ActiveThrottleOperator {
                key: self.key,
                timeout: self.timeout,
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
pub struct ActiveThrottleOperator {
    key: Key,
    timeout: Duration,
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
                OperatorAction::Partial(vec![Emit::key_event(device, key_event.clone())], vec![])
            }
            State::Active => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if Instant::now().duration_since(self.last_emit) > self.timeout {
                    if key_event.key == self.key {
                        // Pressed again before timeout.
                        self.state = State::New;
                        self.on_press(device, key_event)
                    } else {
                        self.state = State::Done;
                        OperatorAction::Done(vec![], vec![Event::key_event2(device, key_event.clone())])
                    }
                } else {
                    if key_event.key == self.key {
                        // Begin squash
                        self.state = State::Squash;
                        OperatorAction::Partial(vec![], vec![])
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
                    OperatorAction::Partial(vec![Emit::key_event(device, key_event.clone())], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    self.state = State::Inactive;
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![], vec![])
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
                    OperatorAction::Partial(vec![Emit::key_event(device, key_event.clone())], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Squash => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Inactive => {
                if key_event.key == self.key {
                    // spurious
                    OperatorAction::Partial(vec![], vec![])
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
