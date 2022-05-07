pub mod action;
pub mod application;
mod key;
pub mod key_action;
pub mod key_press;
pub mod keymap;
mod modmap;

mod remap;
#[cfg(test)]
mod tests;

extern crate serde_yaml;

use keymap::Keymap;
use modmap::Modmap;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use serde::Deserialize;
use std::{error, fs, path::Path, time::SystemTime};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "Vec::new")]
    pub modmap: Vec<Modmap>,
    #[serde(default = "Vec::new")]
    pub keymap: Vec<Keymap>,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(skip)]
    pub modify_time: Option<SystemTime>,
}

pub fn load_config(filename: &Path) -> Result<Config, Box<dyn error::Error>> {
    let yaml = fs::read_to_string(&filename)?;
    let mut config: Config = serde_yaml::from_str(&yaml)?;
    config.modify_time = filename.metadata()?.modified().ok();
    Ok(config)
}

pub fn config_watcher(watch: bool, file: &Path) -> anyhow::Result<Option<Inotify>> {
    if watch {
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
        inotify.add_watch(
            file.parent().expect("config file has a parent directory"),
            AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
        )?;
        inotify.add_watch(file, AddWatchFlags::IN_MODIFY)?;
        Ok(Some(inotify))
    } else {
        Ok(None)
    }
}

fn default_mode() -> String {
    "default".to_string()
}
