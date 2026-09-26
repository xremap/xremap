use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent};
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use crate::KeyValue;
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
    // Waiting for release of trigger key, or interruption.
    // The action has been emitted in this state.
    Pressed,
    Oneshot,
    StandardMod,
    // The trigger key is emitted by local bypass.
    // So must stay active to ensure its release goes same way.
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
                OperatorAction::Partial(
                    vec![],
                    vec![Event::local_bypass(
                        device.clone(),
                        KeyEvent::new(self.action, KeyValue::Press),
                    )],
                )
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
                        vec![],
                        vec![
                            Event::local_bypass(device.clone(), KeyEvent::new(self.action, KeyValue::Release)),
                            // This is bypassed, so it doesn't activate the same operator again.
                            Event::local_bypass(device, key_event.clone()),
                        ],
                    )
                } else {
                    let events = vec![
                        Event::KeyEvent(device.clone(), key_event.clone()),
                        // Releasing after interrupting key.
                        Event::local_bypass(device, KeyEvent::new(self.action, KeyValue::Release)),
                    ];
                    self.state = State::Done;
                    OperatorAction::Done(vec![], events)
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
                    let events = vec![Event::ByPassLocal(Box::new(Event::key_release2(
                        device.clone(),
                        self.action,
                    )))];
                    self.state = State::Done;
                    OperatorAction::Done(vec![], events)
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Cancel => {
                if key_event.key == self.key {
                    self.state = State::Done;
                    OperatorAction::Done(
                        vec![],
                        vec![Event::local_bypass(device, KeyEvent::new(self.key, KeyValue::Release))],
                    )
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
                    OperatorAction::Partial(
                        vec![],
                        vec![Event::local_bypass(
                            device,
                            KeyEvent::new(self.action, KeyValue::Repeat),
                        )],
                    )
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
                    OperatorAction::Partial(
                        vec![],
                        vec![Event::local_bypass(device, KeyEvent::new(self.key, KeyValue::Repeat))],
                    )
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
