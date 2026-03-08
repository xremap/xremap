use crate::config::expmap_operator::ExpmapAction;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::{PRESS, RELEASE, REPEAT};
use crate::operators::{map_actions, ActiveOperator, OperatorAction, StaticOperator};
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use log::error;
use std::mem::swap;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct DoubleTapOperator {
    pub key: Key,
    pub actions: Vec<ExpmapAction>,
    pub timeout: Duration,
    pub timeout_manager: Rc<TimeoutManager>,
}

impl StaticOperator for DoubleTapOperator {
    fn get_operators(&self) -> Vec<(Key, Box<dyn StaticOperator>)> {
        vec![(
            self.key,
            Box::new(DoubleTapOperator {
                key: self.key,
                actions: self.actions.clone(),
                timeout: self.timeout,
                timeout_manager: self.timeout_manager.clone(),
            }),
        )]
    }

    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator> {
        if let Err(err) = self.timeout_manager.set_timeout(self.timeout) {
            error!("Failed to set_timeout: {err}");
        }

        match event {
            Event::KeyEvent(device, key_event) => Box::new(ActiveDoubleTapOperator {
                key: key_event.key,
                actions: self.actions.clone(),
                timeout: self.timeout,
                start_inst: Instant::now(),
                buffered: vec![],
                state: State::Pressed { device: device.clone() },
            }),
            _ => {
                unreachable!()
            }
        }
    }
}

#[derive(Debug)]
enum State {
    Pressed {
        device: Rc<InputDeviceInfo>,
    },
    Tapped {
        press_device: Rc<InputDeviceInfo>,
        release_device: Rc<InputDeviceInfo>,
    },
    Emitted,
    Done,
}

#[derive(Debug)]
pub struct ActiveDoubleTapOperator {
    key: Key,
    actions: Vec<ExpmapAction>,
    timeout: Duration,
    // Time of the first press
    start_inst: Instant,
    buffered: Vec<Event>,
    state: State,
}

impl ActiveOperator for ActiveDoubleTapOperator {
    fn on_event(&mut self, event: &Event) -> OperatorAction {
        match event {
            Event::KeyEvent(device, key_event) => {
                if key_event.value() == PRESS {
                    self.on_press(device.clone(), key_event)
                } else if key_event.value() == RELEASE {
                    self.on_release(device.clone(), key_event)
                } else if key_event.value() == REPEAT {
                    self.on_repeat(device.clone(), key_event)
                } else {
                    // Invalid
                    OperatorAction::Unhandled
                }
            }
            Event::Tick => self.on_tick(),
            _ => self.on_other(event),
        }
    }
}

impl ActiveDoubleTapOperator {
    fn on_press(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::Pressed { device: _ } | State::Emitted if self.key == key_event.key => {
                // Suppress spurious press
                OperatorAction::Undecided
            }
            State::Tapped {
                press_device,
                release_device,
            } if self.key == key_event.key => {
                let emit = map_actions(&self.actions, device, KeyValue::Press);

                // Flush buffered events.
                let mut buffered = vec![];
                swap(&mut buffered, &mut self.buffered);

                self.state = State::Emitted;

                OperatorAction::Partial(emit, buffered)
            }
            // Buffer events when matching
            State::Pressed { device: _ }
            | State::Tapped {
                press_device: _,
                release_device: _,
            } => {
                self.buffered.push(Event::KeyEvent(device.clone(), key_event.clone()));

                OperatorAction::Undecided
            }
            State::Emitted => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_release(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &self.state {
            State::Pressed { device: press_device } if self.key == key_event.key => {
                self.state = State::Tapped {
                    press_device: press_device.clone(),
                    release_device: device.clone(),
                };

                OperatorAction::Undecided
            }
            State::Tapped {
                press_device,
                release_device,
            } if self.key == key_event.key => {
                // Suppress spurious release
                OperatorAction::Undecided
            }
            State::Tapped {
                press_device: _,
                release_device: _,
            }
            | State::Pressed { device: _ } => {
                self.buffered.push(Event::KeyEvent(device.clone(), key_event.clone()));

                OperatorAction::Undecided
            }
            State::Emitted if self.key == key_event.key => {
                self.state = State::Done;
                OperatorAction::Done(map_actions(&self.actions, device, KeyValue::Release), vec![])
            }
            // Unrelated keys not buffered after emit
            State::Emitted => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_repeat(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &self.state {
            // Suppress repeat when matching
            State::Pressed { device: _ }
            | State::Tapped {
                press_device: _,
                release_device: _,
            } => OperatorAction::Undecided,

            // Repeat the emitted key.
            State::Emitted if self.key == key_event.key => {
                OperatorAction::Partial(map_actions(&self.actions, device, KeyValue::Repeat), vec![])
            }

            // Unrelated keys not buffered after emit
            State::Emitted => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_tick(&mut self) -> OperatorAction {
        match &mut self.state {
            State::Pressed { device: _ }
            | State::Tapped {
                press_device: _,
                release_device: _,
            } => {
                if self.start_inst.elapsed() <= self.timeout {
                    OperatorAction::Undecided
                } else {
                    self.state = State::Done;

                    OperatorAction::Cancel
                }
            }
            State::Emitted => {
                // This can happen after emit because timeout isn't cancelled
                OperatorAction::Unhandled
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_other(&mut self, event: &Event) -> OperatorAction {
        match &mut self.state {
            // Suppress when matching
            State::Pressed { device: _ }
            | State::Tapped {
                press_device: _,
                release_device: _,
            } => {
                self.buffered.push(event.clone());
                OperatorAction::Undecided
            }
            State::Emitted => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }
}
