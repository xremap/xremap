use crate::config::deserialize_single_field;
use crate::config::key::parse_key;
use crate::config::key_press::KeyPress;
use crate::config::nested_remap::{deserialize_nested_remap, Remap};
use evdev::KeyCode as Key;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;

// Values in `keymap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeymapAction {
    // Config interface
    KeyPressAndRelease(KeyPress),
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
    WithMark(KeyPress),
    #[serde(deserialize_with = "deserialize_escape_next_key")]
    EscapeNextKey(bool),
    #[serde(deserialize_with = "deserialize_sleep")]
    Sleep(u64),
    #[serde(deserialize_with = "deserialize_close_by_app_class")]
    CloseByAppClass(String),

    // Internals
    #[serde(skip)]
    SetExtraModifiers(Vec<Key>),
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
    use crate::config::key_press::{KeyPress, Modifier};
    use crate::config::keymap_action::{Actions, KeymapAction};
    use evdev::KeyCode as Key;

    #[test]
    fn test_keypress_action() {
        test_yaml_parsing_key_press_and_release(
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

    fn test_yaml_parsing_key_press_and_release(yaml: &str, expected: KeyPress) {
        match serde_yaml::from_str(yaml).unwrap() {
            KeymapAction::KeyPressAndRelease(keyp) => {
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
