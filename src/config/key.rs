use serde::de::Visitor;
use serde::Deserializer;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub fn deserialize_key<'de, D>(deserializer: D) -> Result<evdev::Key, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeyVisitor;

    impl<'de> Visitor<'de> for KeyVisitor {
        type Value = evdev::Key;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(parse_key(value).map_err(serde::de::Error::custom)?)
        }
    }

    deserializer.deserialize_any(KeyVisitor)
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
