use crate::config::key::parse_key;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::error::Error;

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

impl<'de> Deserialize<'de> for KeyPress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key_press = String::deserialize(deserializer)?;
        parse_key_press(&key_press).map_err(serde::de::Error::custom)
    }
}

fn parse_key_press(input: &str) -> Result<KeyPress, Box<dyn Error>> {
    let keys: Vec<&str> = input.split('-').collect();
    if let Some((key, modifier_keys)) = keys.split_last() {
        let mut modifiers = vec![];
        for modifier_key in modifier_keys.iter() {
            modifiers.push(parse_modifier(modifier_key)?);
        }

        Ok(KeyPress {
            key: parse_key(key)?,
            modifiers,
        })
    } else {
        Err(format!("empty key_press: {input}").into())
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

#[test]
fn test_parse_key_press() {
    // Can have modifiers with unspecified sidedness
    assert_eq!(
        parse_key_press("Shift-2").unwrap(),
        KeyPress {
            key: Key::KEY_2,
            modifiers: vec![Modifier::Shift]
        }
    );

    // Can use custom key names. Defined in `parse_key`.
    assert_eq!(
        parse_key_press("Shift_L-2").unwrap(),
        KeyPress {
            key: Key::KEY_2,
            modifiers: vec![Modifier::Key(Key::KEY_LEFTSHIFT)]
        }
    );

    // All keys are accepted as modifiers, because it's not possible to know
    // if the key is listed in virtual_modifiers at this point.
    assert_eq!(
        parse_key_press("Enter-2").unwrap(),
        KeyPress {
            key: Key::KEY_2,
            modifiers: vec![Modifier::Key(Key::KEY_ENTER)]
        }
    );
}
