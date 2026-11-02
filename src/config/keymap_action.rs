use crate::config::deserialize_single_field;
use crate::config::key::parse_key;
use crate::config::key_combo::KeyCombo;
use crate::config::keymap_action_without_args::ActionWithoutArgs;
use crate::config::nested_remap::{deserialize_nested_remap, Remap};
use evdev::KeyCode as Key;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum KeymapAction {
    KeyCombo(KeyCombo),
    KeyPress(Key),
    KeyRepeat(Key),
    KeyRelease(Key),
    Remap(Remap),
    Launch(Vec<String>),
    SetMode(String),
    SetMark(bool),
    WithMark(KeyCombo),
    EscapeNextKey(bool),
    Sleep(u64),
    CloseByAppClass(String),
    Action(ActionWithoutArgs),
}

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ParseKeymapAction {
    // Config interface
    KeyCombo(KeyCombo),
    #[serde(deserialize_with = "deserialize_key_press")]
    KeyPress(Key),
    #[serde(deserialize_with = "deserialize_key_repeat")]
    KeyRepeat(Key),
    #[serde(deserialize_with = "deserialize_key_release")]
    KeyRelease(Key),
    #[serde(deserialize_with = "deserialize_nested_remap")]
    Remap(Remap),
    #[serde(deserialize_with = "deserialize_launch")]
    Launch(Vec<String>),
    #[serde(deserialize_with = "deserialize_set_mode")]
    SetMode(String),
    #[serde(deserialize_with = "deserialize_set_mark")]
    SetMark(bool),
    #[serde(deserialize_with = "deserialize_with_mark")]
    WithMark(KeyCombo),
    #[serde(deserialize_with = "deserialize_escape_next_key")]
    EscapeNextKey(bool),
    #[serde(deserialize_with = "deserialize_sleep")]
    Sleep(u64),
    #[serde(deserialize_with = "deserialize_close_by_app_class")]
    CloseByAppClass(String),
    #[serde(deserialize_with = "deserialize_action_without_args")]
    Action(ActionWithoutArgs),
}

impl ParseKeymapAction {
    pub fn finish(actions: Vec<ParseKeymapAction>) -> Vec<KeymapAction> {
        actions
            .into_iter()
            .flat_map(|action| match action {
                ParseKeymapAction::KeyCombo(value) => vec![KeymapAction::KeyCombo(value)],
                ParseKeymapAction::KeyPress(value) => vec![KeymapAction::KeyPress(value)],
                ParseKeymapAction::KeyRepeat(value) => vec![KeymapAction::KeyRepeat(value)],
                ParseKeymapAction::KeyRelease(value) => vec![KeymapAction::KeyRelease(value)],
                ParseKeymapAction::Remap(value) => vec![KeymapAction::Remap(value)],
                ParseKeymapAction::Launch(value) => vec![KeymapAction::Launch(value)],
                ParseKeymapAction::SetMode(value) => vec![KeymapAction::SetMode(value)],
                ParseKeymapAction::SetMark(value) => vec![KeymapAction::SetMark(value)],
                ParseKeymapAction::WithMark(value) => vec![KeymapAction::WithMark(value)],
                ParseKeymapAction::EscapeNextKey(value) => vec![KeymapAction::EscapeNextKey(value)],
                ParseKeymapAction::Sleep(value) => vec![KeymapAction::Sleep(value)],
                ParseKeymapAction::CloseByAppClass(value) => vec![KeymapAction::CloseByAppClass(value)],
                ParseKeymapAction::Action(value) => vec![KeymapAction::Action(value)],
            })
            .collect()
    }
}

fn deserialize_key_press<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, String>::deserialize(deserializer)?;
    if let Some(key_string) = action.remove("press") {
        if action.is_empty() {
            let key = parse_key(&key_string).map_err(serde::de::Error::custom)?;
            return Ok(key);
        }
    }
    Err(de::Error::custom("not a map with a single \"press\" key"))
}

fn deserialize_key_repeat<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, String>::deserialize(deserializer)?;
    if let Some(key_string) = action.remove("repeat") {
        if action.is_empty() {
            let key = parse_key(&key_string).map_err(serde::de::Error::custom)?;
            return Ok(key);
        }
    }
    Err(de::Error::custom("not a map with a single \"repeat\" key"))
}

fn deserialize_key_release<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, String>::deserialize(deserializer)?;
    if let Some(key_string) = action.remove("release") {
        if action.is_empty() {
            let key = parse_key(&key_string).map_err(serde::de::Error::custom)?;
            return Ok(key);
        }
    }
    Err(de::Error::custom("not a map with a single \"release\" key"))
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

fn deserialize_with_mark<'de, D>(deserializer: D) -> Result<KeyCombo, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, KeyCombo>::deserialize(deserializer)?;
    if let Some(key_combo) = action.remove("with_mark") {
        if action.is_empty() {
            return Ok(key_combo);
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

fn deserialize_sleep<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut action = HashMap::<String, u64>::deserialize(deserializer)?;
    if let Some(set) = action.remove("sleep") {
        if action.is_empty() {
            return Ok(set);
        }
    }
    Err(de::Error::custom("not a map with a single \"sleep\" key"))
}

pub fn deserialize_close_by_app_class<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    deserialize_single_field(deserializer, "close_apps")
}

fn deserialize_action_without_args<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ActionWithoutArgs, D::Error> {
    deserialize_single_field(deserializer, "action")
}
