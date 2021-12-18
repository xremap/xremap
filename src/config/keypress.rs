use crate::config::key::parse_key;
use evdev::Key;
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct KeyPress {
    pub key: Key,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub windows: bool,
}

enum Modifier {
    Shift,
    Control,
    Alt,
    Windows,
}

pub fn parse_keypress(input: &str) -> Result<KeyPress, Box<dyn Error>> {
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
                None => return Err(format!("unknown modifier: {}", modifier).into())
            }
        }

        // TODO: invalidate modifier keys in `key`?
        Ok(KeyPress { key: parse_key(key)?, shift, control, alt, windows })
    } else {
        Err(format!("empty keypress: {}", input).into())
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
