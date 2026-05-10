use crate::config::Config;
use crate::main_controller::MainController;
use anyhow::Result;
use nix::sys::select::FdSet;
use std::path::PathBuf;

pub struct ConfigWatcher {}

impl ConfigWatcher {
    pub fn new(watch: bool, _files: Vec<PathBuf>, _debounce_ms: u64, _notifications: bool) -> Result<Option<Self>> {
        if watch {
            println!("Config watch is not supported on FreeBSD");
        }
        return Ok(None);
    }

    pub fn handle(&mut self, _readable_fds: FdSet, _mainctrl: &mut MainController) -> Result<Option<Config>> {
        unreachable!()
    }
}
