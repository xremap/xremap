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
    #[serde(deserialize_with = "deserialize_launch")]
    Launch(Vec<String>),
    #[serde(deserialize_with = "deserialize_set_mark")]
    SetMark(bool),
    #[serde(deserialize_with = "deserialize_with_mark")]
    WithMark(KeyPress),
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

fn deserialize_launch<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, Vec<String>>::deserialize(deserializer)?;
    if let Some(launch) = action.remove("launch") {
        if action.is_empty() {
            return Ok(launch);
        }
    }
    Err(de::Error::custom("not a map with a single \"launch\" key"))
}

fn deserialize_set_mark<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, bool>::deserialize(deserializer)?;
    if let Some(set) = action.remove("set_mark") {
        if action.is_empty() {
            return Ok(set);
        }
    }
    Err(de::Error::custom("not a map with a single \"set_mark\" key"))
}

fn deserialize_with_mark<'de, D>(deserializer: D) -> Result<KeyPress, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, KeyPress>::deserialize(deserializer)?;
    if let Some(key_press) = action.remove("with_mark") {
        if action.is_empty() {
            return Ok(key_press);
        }
    }
    Err(de::Error::custom("not a map with a single \"with_mark\" key"))
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
