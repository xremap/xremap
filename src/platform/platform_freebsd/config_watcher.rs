use crate::main_impl::MainAction;
use anyhow::Result;
use std::os::fd::RawFd;
use std::path::PathBuf;

pub struct ConfigWatcher {}

impl ConfigWatcher {
    pub fn new(watch: bool, _files: Vec<PathBuf>, _debounce_ms: u64) -> Result<Option<Self>> {
        if watch {
            println!("Config watch is not supported on FreeBSD");
        }
        return Ok(None);
    }

    pub fn handle(&mut self, _readable_fds: Vec<RawFd>) -> Result<Option<MainAction>> {
        unreachable!()
    }
}
