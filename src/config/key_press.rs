use crate::config::key::parse_key;
use evdev::Key;
use serde::{Deserialize, Deserializer};
use std::error;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyPress {
    pub key: Key,
    pub shift: ModifierState,
    pub control: ModifierState,
    pub alt: ModifierState,
    pub windows: ModifierState,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ModifierState {
    Left,
    Right,
    Either,
    None,
}

pub enum Modifier {
    Shift,
    Control,
    Alt,
    Windows,
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
    if let Some((key, modifiers)) = keys.split_last() {
        let mut shift = ModifierState::None;
        let mut control = ModifierState::None;
        let mut alt = ModifierState::None;
        let mut windows = ModifierState::None;

        for modifier in modifiers.iter() {
            match parse_modifier(modifier) {
                Some((Modifier::Shift, state)) => shift = state,
                Some((Modifier::Control, state)) => control = state,
                Some((Modifier::Alt, state)) => alt = state,
                Some((Modifier::Windows, state)) => windows = state,
                None => return Err(format!("unknown modifier: {}", modifier).into()),
            }
        }

        // TODO: invalidate modifier keys in `key`?
        Ok(KeyPress {
            key: parse_key(key)?,
            shift,
            control,
            alt,
            windows,
        })
    } else {
        Err(format!("empty key_press: {}", input).into())
    }
}

fn parse_modifier(modifier: &str) -> Option<(Modifier, ModifierState)> {
    // Everything is case-insensitive
    let mut modifier = &modifier.to_uppercase()[..];
    let mut modifier_state = ModifierState::Either;
    if modifier.ends_with("_L") {
        modifier = remove_suffix(modifier);
        modifier_state = ModifierState::Left;
    } else if modifier.ends_with("_R") {
        modifier = remove_suffix(modifier);
        modifier_state = ModifierState::Right;
    }

    match modifier {
        // Shift
        "SHIFT" => Some((Modifier::Shift, modifier_state)),
        // Control
        "C" => Some((Modifier::Control, modifier_state)),
        "CTRL" => Some((Modifier::Control, modifier_state)),
        "CONTROL" => Some((Modifier::Control, modifier_state)),
        // Alt
        "M" => Some((Modifier::Alt, modifier_state)),
        "ALT" => Some((Modifier::Alt, modifier_state)),
        // Windows
        "SUPER" => Some((Modifier::Windows, modifier_state)),
        "WIN" => Some((Modifier::Windows, modifier_state)),
        "WINDOWS" => Some((Modifier::Windows, modifier_state)),
        // else
        _ => None,
    }
}

fn remove_suffix(string: &str) -> &str {
    let mut chars = string.chars();
    chars.next_back();
    chars.next_back();
    chars.as_str()
}
