use evdev::Key;
use serde::Deserialize;

use crate::config::action::Action;
use crate::config::key_press::KeyPress;
use std::collections::HashMap;
use std::time::Duration;

use super::action::Actions;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub timeout: Option<Duration>,
    pub timeout_key: Option<Key>,
}

// USed only for deserialization
#[derive(Debug, Deserialize)]
pub struct RemapActions {
    pub remap: HashMap<KeyPress, Actions>,
    pub timeout_millis: Option<u64>,
    pub timeout_key: Option<String>,
}
