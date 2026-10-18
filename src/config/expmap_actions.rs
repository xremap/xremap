use crate::config::modmap_operator::Keys;
use crate::device::InputDeviceInfo;
use crate::event::Event;
use crate::KeyValue;
use evdev::KeyCode as Key;
use serde::Deserialize;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ExpmapActions {
    pub keys: Vec<Key>,
    pub bypass: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ParseExpmapActions {
    NoAction,
    Keys(Keys),
}

impl ParseExpmapActions {
    pub fn finish(self, bypass: bool) -> ExpmapActions {
        match self {
            ParseExpmapActions::NoAction => ExpmapActions { keys: vec![], bypass },
            ParseExpmapActions::Keys(keys) => ExpmapActions {
                keys: keys.into_vec(),
                bypass,
            },
        }
    }
}

impl ExpmapActions {
    pub fn emit(&self, device: Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Event> {
        vec![Event::ExpmapActions {
            actions: self.clone(),
            device,
            value,
        }]
    }
}
