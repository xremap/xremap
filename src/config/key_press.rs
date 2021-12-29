use crate::config::key::parse_key;
use evdev::Key;
use serde::de;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::error;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyPress {
    pub key: Key,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub windows: bool,
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
        struct KeyPressVisitor;

        impl<'de> Visitor<'de> for KeyPressVisitor {
            type Value = KeyPress;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_key_press(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(KeyPressVisitor)
    }
}

fn parse_key_press(input: &str) -> Result<KeyPress, Box<dyn error::Error>> {
    let keys: Vec<&str> = input.split("-").collect();
    if let Some((key, modifiers)) = keys.split_last() {
        let mut shift = false;
        let mut control = false;
        let mut alt = false;
        let mut windows = false;

        for modifier in modifiers.iter() {
            match parse_modifier(modifier) {
                Some(Modifier::Shift) => shift = true,
                Some(Modifier::Control) => control = true,
                Some(Modifier::Alt) => alt = true,
                Some(Modifier::Windows) => windows = true,
                None => return Err(format!("unknown modifier: {}", modifier).into()),
            }
        }

        // TODO: invalidate modifier keys in `key`?
        Ok(KeyPress {
            key: Key::new(parse_key(key)?.code()),
            shift,
            control,
            alt,
            windows,
        })
    } else {
        Err(format!("empty key_press: {}", input).into())
    }
}

fn parse_modifier(modifier: &str) -> Option<Modifier> {
    // Everything is case-insensitive
    match &modifier.to_uppercase()[..] {
        // Shift
        "SHIFT" => Some(Modifier::Shift),
        // Control
        "C" => Some(Modifier::Control),
        "CTRL" => Some(Modifier::Control),
        "CONTROL" => Some(Modifier::Control),
        // Alt
        "M" => Some(Modifier::Alt),
        "ALT" => Some(Modifier::Alt),
        // Windows
        "SUPER" => Some(Modifier::Windows),
        "WIN" => Some(Modifier::Windows),
        "WINDOWS" => Some(Modifier::Windows),
        // else
        _ => None,
    }
}
