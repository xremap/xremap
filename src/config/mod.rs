pub mod application;
mod device;
mod key;
pub mod key_press;
pub mod keymap;
pub mod keymap_action;
mod modmap;
pub mod modmap_action;

pub mod remap;
#[cfg(test)]
mod tests;

extern crate serde_yaml;

use evdev::Key;
use keymap::Keymap;
use modmap::Modmap;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, error, fs, path::PathBuf, time::SystemTime};

use self::{
    key::parse_key,
    keymap::{build_keymap_table, KeymapEntry},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Config interface
    #[serde(default = "Vec::new")]
    pub modmap: Vec<Modmap>,
    #[serde(default = "Vec::new")]
    pub keymap: Vec<Keymap>,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(deserialize_with = "deserialize_virtual_modifiers", default = "Vec::new")]
    pub virtual_modifiers: Vec<Key>,
    #[serde(default)]
    pub keypress_delay_ms: u64,

    // Internals
    #[serde(skip)]
    pub modify_time: Option<SystemTime>,
    #[serde(skip)]
    pub keymap_table: HashMap<Key, Vec<KeymapEntry>>,
}

pub fn load_configs(filenames: &Vec<PathBuf>) -> Result<Config, Box<dyn error::Error>> {
    // Assumes filenames is non-empty
    let yaml = fs::read_to_string(&filenames[0])?;
    let mut config: Config = serde_yaml::from_str(&yaml)?;

    for filename in &filenames[1..] {
        let yaml = fs::read_to_string(&filename)?;
        let c: Config = serde_yaml::from_str(&yaml)?;
        config.modmap.extend(c.modmap);
        config.keymap.extend(c.keymap);
        config.virtual_modifiers.extend(c.virtual_modifiers);
    }

    // Timestamp for --watch=config
    config.modify_time = filenames.last().and_then(|path| path.metadata().ok()?.modified().ok());

    // Convert keymap for efficient keymap lookup
    config.keymap_table = build_keymap_table(&config.keymap);

    Ok(config)
}

pub fn config_watcher(watch: bool, files: &Vec<PathBuf>) -> anyhow::Result<Option<Inotify>> {
    if watch {
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
        for file in files {
            inotify.add_watch(
                file.parent().expect("config file has a parent directory"),
                AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
            )?;
            inotify.add_watch(file, AddWatchFlags::IN_MODIFY)?;
        }
        Ok(Some(inotify))
    } else {
        Ok(None)
    }
}

fn default_mode() -> String {
    "default".to_string()
}

fn deserialize_virtual_modifiers<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let key_strs = Vec::<String>::deserialize(deserializer)?;
    let mut keys: Vec<Key> = vec![];
    for key_str in key_strs {
        keys.push(parse_key(&key_str).map_err(serde::de::Error::custom)?);
    }
    return Ok(keys);
}
