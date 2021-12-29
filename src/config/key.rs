use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

// A wrapper of evdev::Key just to ease deserialization
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Key {
    key: evdev::Key,
}

impl Key {
    #[inline]
    pub const fn new(code: u16) -> Self {
        Key {
            key: evdev::Key::new(code),
        }
    }

    #[inline]
    pub const fn code(&self) -> u16 {
        self.key.code()
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Key {
                    key: parse_key(value).map_err(serde::de::Error::custom)?,
                })
            }
        }

        deserializer.deserialize_any(KeyVisitor)
    }
}

pub fn parse_key(input: &str) -> Result<evdev::Key, Box<dyn Error>> {
    // Everything is case-insensitive
    let name = input.to_uppercase();

    // Original evdev scancodes should always work
    if let Ok(key) = evdev::Key::from_str(&name) {
        return Ok(key);
    }

    // You can abbreviate "KEY_" of any "KEY_*" scancodes.
    if let Ok(key) = evdev::Key::from_str(&format!("KEY_{}", name)) {
        return Ok(key);
    }

    // xremap's custom aliases like k0kubun/karabiner-dsl
    let key = match &name[..] {
        // Shift
        "SHIFT_R" => evdev::Key::KEY_RIGHTSHIFT,
        "SHIFT_L" => evdev::Key::KEY_LEFTSHIFT,
        // Control
        "CONTROL_R" => evdev::Key::KEY_RIGHTCTRL,
        "CONTROL_L" => evdev::Key::KEY_LEFTCTRL,
        "CTRL_R" => evdev::Key::KEY_RIGHTCTRL,
        "CTRL_L" => evdev::Key::KEY_LEFTCTRL,
        // Alt
        "ALT_R" => evdev::Key::KEY_RIGHTALT,
        "ALT_L" => evdev::Key::KEY_LEFTALT,
        // Windows
        "SUPER_R" => evdev::Key::KEY_RIGHTMETA,
        "SUPER_L" => evdev::Key::KEY_LEFTMETA,
        "WIN_R" => evdev::Key::KEY_RIGHTMETA,
        "WIN_L" => evdev::Key::KEY_LEFTMETA,
        // else
        _ => evdev::Key::KEY_RESERVED,
    };
    if key != evdev::Key::KEY_RESERVED {
        return Ok(key);
    }

    return Err(format!("unknown key '{}'", input).into());
}
