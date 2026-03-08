use crate::config::expmap_operator::ExpmapAction;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::{PRESS, RELEASE, REPEAT};
use crate::operators::map_actions;
use crate::operators::{ActiveOperator, OperatorAction, StaticOperator};
use crate::timeout_manager::TimeoutManager;
use evdev::KeyCode as Key;
use log::error;
use std::mem::swap;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec;

/// Function
///  - Release action when the first trigger key is released. Alternative
///    is to release when all are released.
///  - Other trigger keys will remain out of sync wrt physical/emitted, this means the operator
///    needs to remain active, so it can filter the physical releases. Alternative is
///    to emit press events for the remaining keys. That might be a good option
///    for modifiers.
///  - Re-emit can only happen after all of the trigger keys have been released.
///    Alternative is to remain active, and if all keys are again down in the future
///    the action will release again. That wouldn't be simultaneous, though.
///  - The device of the last trigger key is used for all emitted actions.
///    And the same device is used for subsequent repeat/release events.

#[derive(Debug)]
pub struct SimOperator {
    pub keys: Vec<Key>,
    pub actions: Vec<ExpmapAction>,
    pub timeout: Duration,
    pub timeout_manager: Rc<TimeoutManager>,
}

impl StaticOperator for SimOperator {
    fn get_operators(&self) -> Vec<(Key, Box<dyn StaticOperator>)> {
        if self.keys.len() < 2 {
            panic!("There must be at least two keys for a chord.");
        }

        // Needs a definition for each start_key.
        self.keys
            .iter()
            .map(|&key| {
                let operator: Box<dyn StaticOperator> = Box::new(SimOperator {
                    keys: self.keys.clone(),
                    actions: self.actions.clone(),
                    timeout: self.timeout,
                    timeout_manager: self.timeout_manager.clone(),
                });

                (key, operator)
            })
            .collect()
    }

    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator> {
        if let Err(err) = self.timeout_manager.set_timeout(self.timeout) {
            error!("Failed to set_timeout: {err}");
        }

        match event {
            Event::KeyEvent(_, key_event) => {
                let still_missing: Vec<_> = self.keys.iter().filter(|&&key| key != key_event.key).cloned().collect();

                Box::new(ActiveSimOperator {
                    keys: self.keys.clone(),
                    actions: self.actions.clone(),
                    start_inst: Instant::now(),
                    buffered: vec![],
                    state: State::Pressed { still_missing },
                    timeout: self.timeout,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}

#[derive(Debug)]
enum State {
    // Some trigger keys have been pressed.
    Pressed { still_missing: Vec<Key> },
    // The action has been pressed.
    Emitted { device: Rc<InputDeviceInfo> },
    // The action has been released, and release of triggers will be squashed.
    Released { still_pressed: Vec<Key> },
    // All trigger keys have been released again.
    // Or the operator was canceled before emit.
    Done,
}

#[derive(Debug)]
pub struct ActiveSimOperator {
    keys: Vec<Key>,
    actions: Vec<ExpmapAction>,
    timeout: Duration,
    // Time of the first press
    start_inst: Instant,
    buffered: Vec<Event>,
    state: State,
}

impl ActiveOperator for ActiveSimOperator {
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
                    OperatorAction::Unhandled
                }
            }
            Event::Tick => self.on_tick(),
            _ => self.on_other(event),
        }
    }
}

impl ActiveSimOperator {
    fn on_press(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::Pressed { still_missing } => {
                if vec![key_event.key] == *still_missing {
                    // All keys pressed
                    let emit = map_actions(&self.actions, device.clone(), KeyValue::Press);

                    // Flush buffered events.
                    let mut buffered = vec![];
                    swap(&mut buffered, &mut self.buffered);

                    self.state = State::Emitted { device: device.clone() };

                    OperatorAction::Partial(emit, buffered)
                } else if still_missing.contains(&key_event.key) {
                    // One more trigger key pressed, but not all, yet.

                    still_missing.retain(|&key| key != key_event.key);

                    OperatorAction::Undecided
                } else {
                    self.buffered.push(Event::KeyEvent(device.clone(), key_event.clone()));

                    OperatorAction::Undecided
                }
            }
            State::Emitted { device: _ } => {
                debug_assert!(self.buffered.is_empty());

                if self.keys.contains(&key_event.key) {
                    // All keys are pressed in this state, so can't be pressed again.
                    unreachable!()
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Released { still_pressed: _ } => {
                debug_assert!(self.buffered.is_empty());

                // Press is ignored in this state. The only purpose is to squash
                // release of trigger keys.
                OperatorAction::Unhandled
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_release(&mut self, device: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &mut self.state {
            State::Pressed { still_missing } => {
                if self.keys.contains(&key_event.key) && !still_missing.contains(&key_event.key) {
                    // A trigger key has been pressed and now released. So cancel.
                    self.state = State::Done;

                    OperatorAction::Cancel
                } else {
                    self.buffered.push(Event::KeyEvent(device.clone(), key_event.clone()));

                    OperatorAction::Undecided
                }
            }
            State::Emitted { device } => {
                debug_assert!(self.buffered.is_empty());

                if self.keys.contains(&key_event.key) {
                    let emit = map_actions(&self.actions, device.clone(), KeyValue::Release);

                    let still_pressed: Vec<_> =
                        self.keys.iter().filter(|&&key| key != key_event.key).cloned().collect();

                    self.state = State::Released { still_pressed };

                    OperatorAction::Partial(emit, vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Released { still_pressed } => {
                debug_assert!(self.buffered.is_empty());

                if vec![key_event.key] == *still_pressed {
                    // All released
                    self.state = State::Done;

                    OperatorAction::Done(vec![], vec![])
                } else if still_pressed.contains(&key_event.key) {
                    // To squash
                    still_pressed.retain(|&key| key != key_event.key);

                    OperatorAction::Partial(vec![], vec![])
                } else if self.keys.contains(&key_event.key) && !still_pressed.contains(&key_event.key) {
                    // Already squashed, but now released again.
                    OperatorAction::Unhandled
                } else {
                    // Unrelated
                    OperatorAction::Unhandled
                }
            }
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_repeat(&mut self, _: Rc<InputDeviceInfo>, key_event: &KeyEvent) -> OperatorAction {
        match &self.state {
            // Suppress repeat when matching
            State::Pressed { still_missing: _ } => OperatorAction::Undecided,
            State::Emitted { device } => {
                if self.keys[0] == key_event.key {
                    // Only one trigger key must be repeated, otherwise would repeat
                    // events be multiplied for the action.
                    // Maybe this should be the last key pressed, because that might be the
                    // only one, that sends repeat signals.
                    OperatorAction::Partial(map_actions(&self.actions, device.clone(), KeyValue::Repeat), vec![])
                } else if self.keys.contains(&key_event.key) {
                    // These are unneeded
                    OperatorAction::Partial(vec![], vec![])
                } else {
                    OperatorAction::Unhandled
                }
            }
            State::Released { still_pressed } => {
                if still_pressed.contains(&key_event.key) {
                    // It's still squashed, because it hasn't been released yet.
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
        match &self.state {
            State::Pressed { still_missing: _ } if self.start_inst.elapsed() <= self.timeout => {
                OperatorAction::Undecided
            }
            State::Pressed { still_missing: _ } => {
                self.state = State::Done;

                OperatorAction::Cancel
            }
            State::Released { still_pressed: _ } => OperatorAction::Unhandled,
            State::Emitted { device: _ } => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }

    fn on_other(&mut self, event: &Event) -> OperatorAction {
        match &mut self.state {
            State::Pressed { still_missing: _ } => {
                self.buffered.push(event.clone());
                OperatorAction::Undecided
            }
            State::Emitted { device: _ } | State::Released { still_pressed: _ } => OperatorAction::Unhandled,
            State::Done => {
                unreachable!()
            }
        }
    }
}
