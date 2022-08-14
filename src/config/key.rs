use evdev::Key;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::str::FromStr;

pub fn deserialize_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let key = String::deserialize(deserializer)?;
    parse_key(&key).map_err(serde::de::Error::custom)
}

pub fn parse_key(input: &str) -> Result<Key, Box<dyn Error>> {
    // Everything is case-insensitive
    let name = input.to_uppercase();

    // Original evdev scancodes should always work
    if let Ok(key) = Key::from_str(&name) {
        return Ok(key);
    }

    // You can abbreviate "KEY_" of any "KEY_*" scancodes.
    if let Ok(key) = Key::from_str(&format!("KEY_{}", name)) {
        return Ok(key);
    }

    // xremap's custom aliases like k0kubun/karabiner-dsl
    let key = match &name[..] {
        // Shift
        "SHIFT_R" => Key::KEY_RIGHTSHIFT,
        "SHIFT_L" => Key::KEY_LEFTSHIFT,
        // Control
        "CONTROL_R" => Key::KEY_RIGHTCTRL,
        "CONTROL_L" => Key::KEY_LEFTCTRL,
        "CTRL_R" => Key::KEY_RIGHTCTRL,
        "CTRL_L" => Key::KEY_LEFTCTRL,
        "C_R" => Key::KEY_RIGHTCTRL,
        "C_L" => Key::KEY_LEFTCTRL,
        // Alt
        "ALT_R" => Key::KEY_RIGHTALT,
        "ALT_L" => Key::KEY_LEFTALT,
        "M_R" => Key::KEY_RIGHTALT,
        "M_L" => Key::KEY_LEFTALT,
        // Windows
        "SUPER_R" => Key::KEY_RIGHTMETA,
        "SUPER_L" => Key::KEY_LEFTMETA,
        "WIN_R" => Key::KEY_RIGHTMETA,
        "WIN_L" => Key::KEY_LEFTMETA,
        // else
        _ => Key::KEY_RESERVED,
    };
    if key != Key::KEY_RESERVED {
        return Ok(key);
    }

    return Err(format!("unknown key '{}'", input).into());
}
