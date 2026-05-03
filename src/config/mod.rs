pub mod application;
pub mod device;
mod expmap;
pub mod expmap_operator;
pub mod expmap_simkey;
mod key;
pub mod key_press;
pub mod keymap;
pub mod keymap_action;
mod modmap;
pub mod modmap_operator;
pub mod nested_remap;
#[cfg(test)]
mod tests;
mod validation;
mod watcher;

pub use crate::config::expmap::Expmap;
use crate::config::key::parse_key;
use crate::config::keymap::{build_keymap_table, KeymapEntry};
use crate::event_handler::DISGUISED_EVENT_OFFSETTER;
use crate::event_handler::MODIFIER_KEYS;
use evdev::KeyCode as Key;
use keymap::Keymap;
use modmap::Modmap;
use serde::{de::IgnoredAny, Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{error, fs};
pub use validation::validate_config_file;
pub use watcher::ConfigWatcher;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Config interface
    #[serde(default = "Vec::new")]
    pub experimental_map: Vec<Expmap>,
    #[serde(default = "Vec::new")]
    pub modmap: Vec<Modmap>,
    #[serde(default = "Vec::new")]
    pub keymap: Vec<Keymap>,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(deserialize_with = "deserialize_virtual_modifier", default = "Vec::new")]
    pub virtual_modifiers: Vec<Key>,
    #[serde(default)]
    pub keypress_delay_ms: u64,
    #[serde(default)]
    pub throttle_ms: u64,
    #[serde(default)]
    pub config_watch_debounce_ms: u64,
    #[serde(default)]
    pub notifications: bool,

    // Data is not used by any part of the application.
    // but can be used with Anchors and Aliases
    #[allow(dead_code)]
    #[serde(default)]
    pub shared: IgnoredAny,

    // Internals
    #[serde(skip)]
    pub modify_time: Option<SystemTime>,
    #[serde(skip)]
    pub keymap_table: HashMap<Key, Vec<KeymapEntry>>,
    #[serde(default = "const_true")]
    pub enable_wheel: bool,
}

enum ConfigFiletype {
    Yaml,
    Toml,
}

fn get_file_ext(filename: &Path) -> ConfigFiletype {
    match filename.extension() {
        Some(f) => {
            if f.to_str().unwrap_or("").to_lowercase() == "toml" {
                ConfigFiletype::Toml
            } else {
                ConfigFiletype::Yaml
            }
        }
        _ => ConfigFiletype::Yaml,
    }
}

pub fn load_configs(filenames: &[PathBuf]) -> Result<Config, Box<dyn error::Error>> {
    assert!(!filenames.is_empty(), "config is set, if not completions");

    // Assumes filenames is non-empty
    let config_contents = fs::read_to_string(&filenames[0])?;

    let mut config: Config = match get_file_ext(&filenames[0]) {
        ConfigFiletype::Yaml => serde_yaml::from_str(&config_contents)?,
        ConfigFiletype::Toml => toml::from_str(&config_contents)?,
    };

    for filename in &filenames[1..] {
        let config_contents = fs::read_to_string(filename)?;
        let c: Config = match get_file_ext(filename) {
            ConfigFiletype::Yaml => serde_yaml::from_str(&config_contents)?,
            ConfigFiletype::Toml => toml::from_str(&config_contents)?,
        };

        config.modmap.extend(c.modmap);
        config.keymap.extend(c.keymap);
        config.virtual_modifiers.extend(c.virtual_modifiers);
    }

    // Timestamp for --watch=config
    config.modify_time = filenames.last().and_then(|path| path.metadata().ok()?.modified().ok());

    // Convert keymap for efficient keymap lookup
    config.keymap_table = build_keymap_table(&config.keymap);

    validate_config_file(&config)?;

    Ok(config)
}

fn default_mode() -> String {
    "default".to_string()
}

fn deserialize_keys<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let key_strs = Vec::<String>::deserialize(deserializer)?;
    let mut keys: Vec<Key> = vec![];
    for key_str in key_strs {
        keys.push(parse_key(&key_str).map_err(serde::de::Error::custom)?);
    }
    Ok(keys)
}

fn deserialize_virtual_modifier<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let key_strs = Vec::<String>::deserialize(deserializer)?;
    let mut keys: Vec<Key> = vec![];
    for key_str in key_strs {
        let key = parse_key(&key_str).map_err(serde::de::Error::custom)?;
        if MODIFIER_KEYS.contains(&key) {
            return Err(serde::de::Error::custom(format!("Can't use '{key_str}' as virtual modifier")));
        }
        if key.code() >= DISGUISED_EVENT_OFFSETTER {
            return Err(serde::de::Error::custom(format!(
                "Can't use a relative-event ({key_str}) as virtual modifier"
            )));
        }
        keys.push(key);
    }
    Ok(keys)
}

fn const_true() -> bool {
    true
}

pub fn deserialize_single_field<'de, D, T>(deserializer: D, name: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let mut map = HashMap::<String, T>::deserialize(deserializer)?;

    if let Some(value) = map.remove(name) {
        if map.is_empty() {
            return Ok(value);
        }
    }

    Err(serde::de::Error::custom(format!("This error is never shown in an untagged enum")))
}
