use crate::config::application::ApplicationMatch;
use crate::config::expmap_operator::ExpmapAction;
use crate::device::InputDeviceInfo;
use crate::emit_handler::Emit;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::{PRESS, RELEASE, REPEAT};
use std::fmt::Debug;
use std::rc::Rc;

pub trait StaticOperator: Debug {
    // Return a candidate when the start_key is pressed.
    fn get_active_operator(&self, event: &Event) -> Box<dyn ActiveOperator>;
}

#[derive(Debug, Clone)]
pub enum OperatorAction {
    // The operator is still buffering events, and may or may not match.
    Undecided,
    // The operator does not match.
    // This must only be emitted if only Undecided events have been emitted.
    Cancel,
    // The event is unhandled by this operator, and must go to next operators.
    Unhandled,
    // The operator consumes the event and remains active.
    // 1st vector to next level
    // 2nd vector to next operators
    Partial(Vec<Emit>, Vec<Event>),
    // The operator is done and asks to be removed
    // 1st vector to next level
    // 2nd vector to next operators
    Done(Vec<Emit>, Vec<Event>),
}

pub trait ActiveOperator: Debug {
    /// Either implement this method, or each of the methods called by this one.
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

    fn on_press(&mut self, _device: Rc<InputDeviceInfo>, _key_event: &KeyEvent) -> OperatorAction {
        unreachable!()
    }

    fn on_release(&mut self, _device: Rc<InputDeviceInfo>, _key_event: &KeyEvent) -> OperatorAction {
        unreachable!()
    }

    fn on_repeat(&mut self, _device: Rc<InputDeviceInfo>, _key_event: &KeyEvent) -> OperatorAction {
        unreachable!()
    }

    fn on_tick(&mut self) -> OperatorAction {
        unreachable!()
    }

    fn on_other(&mut self, _event: &Event) -> OperatorAction {
        unreachable!()
    }
}

pub fn map_actions(actions: &Vec<ExpmapAction>, device: Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Emit> {
    actions
        .iter()
        .filter_map(|action| match action {
            ExpmapAction::Key(key) => Some(Emit::key_event(device.clone(), KeyEvent::new(*key, value))),
        })
        .collect()
}

// Internals for efficient operator lookup
#[derive(Debug)]
pub struct OperatorEntry {
    pub operator: Box<dyn StaticOperator>,
    pub application: Option<ApplicationMatch>,
    pub title: Option<ApplicationMatch>,
}
