use crate::config::expmap_operator::ExpmapAction;
use crate::device::InputDeviceInfo;
use crate::emit_handler::Emit;
use crate::event::{Event, KeyEvent, KeyValue};
use evdev::KeyCode as Key;
use std::fmt::Debug;
use std::rc::Rc;

pub trait StaticOperator: Debug {
    // To allow operators to have more than one start_key.
    fn get_operators(&self) -> Vec<(Key, Box<dyn StaticOperator>)>;
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
    fn on_event(&mut self, event: &Event) -> OperatorAction;
}

pub fn map_actions(actions: &Vec<ExpmapAction>, device: Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Emit> {
    actions
        .iter()
        .filter_map(|action| match action {
            ExpmapAction::Key(key) => Some(Emit::key_event(device.clone(), KeyEvent::new(*key, value))),
        })
        .collect()
}
