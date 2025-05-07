use evdev::KeyCode as Key;
use serde::Deserialize;

use crate::config::key_press::KeyPress;
use crate::config::keymap_action::KeymapAction;
use std::collections::HashMap;
use std::time::Duration;

use super::keymap_action::Actions;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: HashMap<KeyPress, Vec<KeymapAction>>,
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
