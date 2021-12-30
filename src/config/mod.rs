pub mod action;
pub mod application;
mod key;
pub mod key_action;
pub mod key_press;
mod hotkey;
mod keymap;
mod modmap;

#[cfg(test)]
mod tests;

extern crate serde_yaml;

use keymap::Keymap;
use modmap::Modmap;
use hotkey::Hotkey;
use serde::Deserialize;
use std::{error, fs};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "Vec::new")]
    pub modmap: Vec<Modmap>,
    #[serde(default = "Vec::new")]
    pub keymap: Vec<Keymap>,
    #[serde(default = "Vec::new")]
    pub hotkeys: Vec<Hotkey>,
}

pub fn load_config(filename: &str) -> Result<Config, Box<dyn error::Error>> {
    let yaml = fs::read_to_string(&filename)?;
    let config: Config = serde_yaml::from_str(&yaml)?;
    return Ok(config);
}
