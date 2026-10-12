use crate::config::modmap_operator::Keys;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent};
use crate::KeyValue;
use evdev::KeyCode as Key;
use serde::Deserialize;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ExpmapActions {
    pub keys: Vec<Key>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ParseExpmapActions {
    NoAction,
    Keys(Keys),
}

impl ParseExpmapActions {
    pub fn finish(self) -> ExpmapActions {
        match self {
            ParseExpmapActions::NoAction => ExpmapActions { keys: vec![] },
            ParseExpmapActions::Keys(keys) => ExpmapActions { keys: keys.into_vec() },
        }
    }
}

impl ExpmapActions {
    pub fn emit(&self, device: Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Event> {
        self.keys
            .iter()
            .filter_map(|key| Some(Event::local_bypass(device.clone(), KeyEvent::new(*key, value))))
            .collect()
    }
}
