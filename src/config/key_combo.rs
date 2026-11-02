use crate::config::{key::parse_key, keymap_action::KeymapAction};
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::error::Error;

#[derive(Copy, Clone)]
pub enum EmitMode {
    #[allow(unused)]
    Key,
    Combo,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyCombo {
    pub key: Key,
    pub modifiers: Vec<Modifier>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
// This type essentially just ensures, that EmitMode is
// considered for all ParseCombos.
pub struct ParseCombo {
    pub key: Key,
    pub modifiers: Vec<Modifier>,
}

impl ParseCombo {
    pub fn as_is(self) -> KeyCombo {
        KeyCombo {
            key: self.key,
            modifiers: self.modifiers,
        }
    }

    pub fn finish(self, mode: EmitMode) -> Vec<KeymapAction> {
        if self.modifiers.is_empty() {
            match mode {
                EmitMode::Key => {
                    // Default key-tap when no explicit modifiers.
                    vec![KeymapAction::KeyPress(self.key), KeymapAction::KeyRelease(self.key)]
                }
                EmitMode::Combo => {
                    // Default key-combo when no explicit modifiers.
                    vec![KeymapAction::KeyCombo(self.as_is())]
                }
            }
        } else {
            // If user has chosen explicit modifiers, it has to be a key combo.
            vec![KeymapAction::KeyCombo(self.as_is())]
        }
    }
}

impl<'de> Deserialize<'de> for ParseCombo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key_combo = String::deserialize(deserializer)?;
        parse_key_combo(&key_combo).map_err(serde::de::Error::custom)
    }
}

pub fn parse_key_combo(input: &str) -> Result<ParseCombo, Box<dyn Error>> {
    let keys: Vec<&str> = input.split('-').collect();
    if let Some((key, modifier_keys)) = keys.split_last() {
        let mut modifiers = vec![];
        for modifier_key in modifier_keys.iter() {
            modifiers.push(parse_modifier(modifier_key)?);
        }

        Ok(ParseCombo {
            key: parse_key(key)?,
            modifiers,
        })
    } else {
        Err(format!("empty key_combo: {input}").into())
    }
}

fn parse_modifier(modifier: &str) -> Result<Modifier, Box<dyn Error>> {
    match parse_modifier_alias(modifier) {
        Some(modifier) => Ok(modifier),
        None => {
            // Modifier by the precise key to use
            parse_key(modifier).map(Modifier::Key)
        }
    }
}

// Modifier that can match both left and right variants.
pub fn parse_modifier_alias(modifier: &str) -> Option<Modifier> {
    // Everything is case-insensitive
    match &modifier.to_uppercase()[..] {
        // Shift
        "S" => Some(Modifier::Shift),
        "SHIFT" => Some(Modifier::Shift),
        // Control
        "C" => Some(Modifier::Control),
        "CTRL" => Some(Modifier::Control),
        "CONTROL" => Some(Modifier::Control),
        // Alt
        "A" => Some(Modifier::Alt),
        "M" => Some(Modifier::Alt),
        "ALT" => Some(Modifier::Alt),
        // Windows
        "SUPER" => Some(Modifier::Windows),
        "W" => Some(Modifier::Windows),
        "WIN" => Some(Modifier::Windows),
        "WINDOWS" => Some(Modifier::Windows),
        _ => None,
    }
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

impl Modifier {
    pub fn is_in(&self, modifiers: &Vec<Key>) -> bool {
        match self {
            Modifier::Shift => modifiers.contains(&Key::KEY_LEFTSHIFT) || modifiers.contains(&Key::KEY_RIGHTSHIFT),
            Modifier::Control => modifiers.contains(&Key::KEY_LEFTCTRL) || modifiers.contains(&Key::KEY_RIGHTCTRL),
            Modifier::Alt => modifiers.contains(&Key::KEY_LEFTALT) || modifiers.contains(&Key::KEY_RIGHTALT),
            Modifier::Windows => modifiers.contains(&Key::KEY_LEFTMETA) || modifiers.contains(&Key::KEY_RIGHTMETA),
            Modifier::Key(key) => modifiers.contains(key),
        }
    }
}
