use crate::config::key_press::KeyPress;
use std::collections::HashMap;

use crate::config::keymap::deserialize_remap;
use serde::Deserialize;
use std::fmt::Debug;

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
pub enum Action {
    KeyPress(KeyPress),
    #[serde(deserialize_with = "deserialize_remap")]
    Remap(HashMap<KeyPress, Vec<Action>>),
}
