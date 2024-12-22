use crate::config::key_press::KeyPress;
use std::collections::HashMap;

use crate::config::remap::Remap;
use evdev::Key;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::time::Duration;

use super::key::parse_key;
use super::remap::RemapActions;

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeymapAction {
    // Config interface
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

    // Internals
    #[serde(skip)]
    SetExtraModifiers(Vec<Key>),
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<Remap, D::Error>
where
    D: Deserializer<'de>,
{
    let action = RemapActions::deserialize(deserializer)?;
    Ok(Remap {
        remap: action.remap.into_iter().map(|(k, v)| (k, v.into_vec())).collect(),
        timeout: action.timeout_millis.map(Duration::from_millis),
        timeout_key: if let Some(key) = action.timeout_key {
            match parse_key(&key) {
                Ok(key) => Some(key),
                Err(e) => return Err(de::Error::custom(e.to_string())),
            }
        } else {
            None
        },
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
    // Allows keychords to map to null, which means no actions.
    NoAction,
    Action(KeymapAction),
    Actions(Vec<KeymapAction>),
}

impl Actions {
    pub fn into_vec(self) -> Vec<KeymapAction> {
        match self {
            Actions::NoAction => vec![],
            Actions::Action(action) => vec![action],
            Actions::Actions(actions) => actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeymapAction;
    use crate::config::key_press::KeyPress;
    use crate::config::key_press::Modifier;
    use crate::config::keymap_action::Actions;
    use evdev::Key;

    #[test]
    fn test_keypress_action() {
        test_yaml_parsing_key_press(
            "c-x",
            KeyPress {
                key: Key::KEY_X,
                modifiers: vec![Modifier::Control],
            },
        );
    }

    #[test]
    fn test_launch_action() {
        test_yaml_parsing_key_launch("{launch: []}", vec![]);
        test_yaml_parsing_key_launch("{launch: [\"bla\"]}", vec!["bla".into()]);
    }

    #[test]
    fn test_null_action() {
        if let Actions::NoAction = serde_yaml::from_str("null").unwrap() {
            return;
        }
        panic!("unexpected type");
    }

    //
    // util
    //

    fn test_yaml_parsing_key_press(yaml: &str, expected: KeyPress) {
        match serde_yaml::from_str(yaml).unwrap() {
            KeymapAction::KeyPress(keyp) => {
                assert_eq!(keyp, expected);
            }
            _ => panic!("unexpected type"),
        }
    }

    fn test_yaml_parsing_key_launch(yaml: &str, expected: Vec<String>) {
        match serde_yaml::from_str(yaml).unwrap() {
            KeymapAction::Launch(vect) => {
                assert_eq!(vect, expected);
            }
            _ => panic!("unexpected type"),
        }
    }
}
