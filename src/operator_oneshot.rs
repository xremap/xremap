use crate::device::InputDeviceInfo;
use crate::emit_handler::Emit;
use crate::event::{Event, KeyEvent};
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use evdev::KeyCode as Key;
use std::rc::Rc;
use std::vec;

#[derive(Debug, Clone)]
pub struct OneshotOperator {
    key: Key,
    action: Key,
}

impl OneshotOperator {
    pub fn get_ops(key: Key, action: Key) -> Vec<(Key, Box<dyn StaticOperator>)> {
        vec![(key, Box::new(OneshotOperator { key, action }))]
    }
}

impl StaticOperator for OneshotOperator {
    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator> {
        match event {
            Event::KeyEvent(_, _) => Box::new(ActiveOneshotOperator {
                key: self.key,
                action: self.action.clone(),
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
    Pressed,
    Oneshot,
    StandardMod,
    Cancel,
    Done,
}

#[derive(Debug)]
pub struct ActiveOneshotOperator {
    key: Key,
    action: Key,
    state: State,
}

impl ActiveOperator for ActiveOneshotOperator {
    fn on_press(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::New => {
                self.state = State::Pressed;
                OperatorAction::Partial(vec![Emit::key_press(device.clone(), self.action)], vec![])
            }
            State::Pressed => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    self.state = State::StandardMod;
                    OperatorAction::Unhandled
                }
            }
            State::Oneshot => {
                if key_event.key == self.key {
                    // Cancel because it's repressed.
                    self.state = State::Cancel;
                    OperatorAction::Partial(
                        vec![
                            Emit::key_release(device.clone(), self.action),
                            // This is emitted, so it doesn't activate the same operator again.
                            Emit::key_event(device.clone(), key_event.clone()),
                        ],
                        vec![],
                    )
                } else {
                    let unhandled = vec![
                        Event::KeyEvent(device.clone(), key_event.clone()),
                        // Releasing after interrupting key, means it must go in unhandled array.
                        Event::ByPassLocal(Box::new(Event::key_release2(device.clone(), self.action))),
                    ];
                    self.state = State::Done;
                    OperatorAction::Done(vec![], unhandled)
                }
            }
            State::StandardMod => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    // Normal key
                    OperatorAction::Unhandled
                }
            }
            State::Cancel => {
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

    fn on_release(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::New => unreachable!(),
            State::Pressed => {
                if key_event.key == self.key {
                    // Delay action-release.
                    self.state = State::Oneshot;
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    // Release doesn't interrupt.
                    OperatorAction::Unhandled
                }
            }
            State::Oneshot => {
                if key_event.key == self.key {
                    // Spurious is suppressed
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    // Release doesn't consume the oneshot.
                    OperatorAction::Unhandled
                }
            }
            State::StandardMod => {
                if key_event.key == self.key {
                    self.state = State::Done;
                    OperatorAction::Done(vec![Emit::key_release(device.clone(), self.action)], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Cancel => {
                if key_event.key == self.key {
                    self.state = State::Done;
                    OperatorAction::Done(vec![Emit::key_release(device.clone(), self.key)], vec![])
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
            State::Pressed => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![Emit::key_repeat(device.clone(), self.action)], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Oneshot => {
                if key_event.key == self.key {
                    // Spurious
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::StandardMod => {
                if key_event.key == self.key {
                    // Spurious
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Cancel => {
                if key_event.key == self.key {
                    OperatorAction::Partial(vec![Emit::key_repeat(device.clone(), self.key)], vec![])
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
