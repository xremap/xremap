use crate::config::key_press::KeyPress;
use std::collections::HashMap;

use crate::config::remap::Remap;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::time::Duration;

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Action {
    KeyPress(KeyPress),
    #[serde(deserialize_with = "deserialize_remap")]
    Remap(Remap),
    #[serde(deserialize_with = "deserialize_launch")]
    Launch(Vec<String>),
    #[serde(deserialize_with = "deserialize_set_mode")]
    SetMode(String),
    #[serde(deserialize_with = "deserialize_set_mark")]
    SetMark(bool),
    #[serde(deserialize_with = "deserialize_with_mark")]
    WithMark(KeyPress),
    #[serde(deserialize_with = "deserialize_escape_next_key")]
    EscapeNextKey(bool),
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<Remap, D::Error>
where
    D: Deserializer<'de>,
{
    let action = RemapActions::deserialize(deserializer)?;
    Ok(Remap {
        remap: action.remap.into_iter().map(|(k, v)| (k, v.into_vec())).collect(),
        timeout: action.timeout_millis.map(Duration::from_millis),
    })
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

fn deserialize_set_mode<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, String>::deserialize(deserializer)?;
    if let Some(set) = action.remove("set_mode") {
        if action.is_empty() {
            return Ok(set);
        }
    }
    Err(de::Error::custom("not a map with a single \"set_mode\" key"))
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

fn deserialize_escape_next_key<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, bool>::deserialize(deserializer)?;
    if let Some(set) = action.remove("escape_next_key") {
        if action.is_empty() {
            return Ok(set);
        }
    }
    Err(de::Error::custom("not a map with a single \"escape_next_key\" key"))
}

// Used only for deserializing Vec<Action>
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Actions {
    Action(Action),
    Actions(Vec<Action>),
}

impl Actions {
    pub fn into_vec(self) -> Vec<Action> {
        match self {
            Actions::Action(action) => vec![action],
            Actions::Actions(actions) => actions,
        }
    }
}

// Used only for deserializing Remap with Vec<Action>
#[derive(Debug, Deserialize)]
pub struct RemapActions {
    pub remap: HashMap<KeyPress, Actions>,
    pub timeout_millis: Option<u64>,
}
