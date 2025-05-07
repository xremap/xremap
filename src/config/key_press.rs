use crate::config::key::parse_key;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::error::{self, Error};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyPress {
    pub key: Key,
    pub modifiers: Vec<Modifier>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Modifier {
    // Matches left, right, or both
    Shift,
    Control,
    Alt,
    Windows,
    // Matches exactly this key
    Key(Key),
}

impl<'de> Deserialize<'de> for KeyPress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key_press = String::deserialize(deserializer)?;
        parse_key_press(&key_press).map_err(serde::de::Error::custom)
    }
}

fn parse_key_press(input: &str) -> Result<KeyPress, Box<dyn error::Error>> {
    let keys: Vec<&str> = input.split('-').collect();
    if let Some((key, modifier_keys)) = keys.split_last() {
        let mut modifiers = vec![];
        for modifier_key in modifier_keys.iter() {
            match parse_modifier(modifier_key) {
                Ok(modifier) => modifiers.push(modifier),
                Err(e) => return Err(e),
            }
        }

        Ok(KeyPress {
            key: parse_key(key)?,
            modifiers,
        })
    } else {
        Err(format!("empty key_press: {}", input).into())
    }
}

fn parse_modifier(modifier: &str) -> Result<Modifier, Box<dyn Error>> {
    // Everything is case-insensitive
    match &modifier.to_uppercase()[..] {
        // Shift
        "SHIFT" => Ok(Modifier::Shift),
        // Control
        "C" => Ok(Modifier::Control),
        "CTRL" => Ok(Modifier::Control),
        "CONTROL" => Ok(Modifier::Control),
        // Alt
        "M" => Ok(Modifier::Alt),
        "ALT" => Ok(Modifier::Alt),
        // Windows
        "SUPER" => Ok(Modifier::Windows),
        "WIN" => Ok(Modifier::Windows),
        "WINDOWS" => Ok(Modifier::Windows),
        // else
        key => parse_key(key).map(|key| Modifier::Key(key)),
    }
}
