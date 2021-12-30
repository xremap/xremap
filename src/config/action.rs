use crate::config::key_press::KeyPress;
use std::collections::HashMap;

use serde::de;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Action {
    KeyPress(KeyPress),
    #[serde(deserialize_with = "deserialize_remap")]
    Remap(HashMap<KeyPress, Vec<Action>>),
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<Action>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, HashMap<KeyPress, Actions>>::deserialize(deserializer)?;
    if let Some(remap) = action.remove("remap") {
        if action.is_empty() {
            return Ok(remap.into_iter().map(|(k, v)| (k, v.to_vec())).collect());
        }
    }
    Err(de::Error::custom("not a map with a single \"remap\" key"))
}

// Used only for deserializing Vec<Action>
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Actions {
    Action(Action),
    Actions(Vec<Action>),
}

impl Actions {
    pub fn to_vec(self) -> Vec<Action> {
        match self {
            Actions::Action(action) => vec![action],
            Actions::Actions(actions) => actions,
        }
    }
}
