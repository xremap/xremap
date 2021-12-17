use crate::config::key::parse_key;
use evdev::Key;
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct KeyPress {
    pub key: Key,
}

pub fn parse_keypress(input: &str) -> Result<KeyPress, Box<dyn Error>> {
    let key = parse_key(input)?;
    return Ok(KeyPress { key });
}
