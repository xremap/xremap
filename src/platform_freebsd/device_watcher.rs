use std::path::PathBuf;

pub struct DeviceWatcher {}

impl DeviceWatcher {
    pub fn new(watch: bool) -> anyhow::Result<Option<Self>> {
        if watch {
            println!("Device watch is not supported on FreeBSD");
        }
        Ok(None)
    }

    pub fn read_events(&self) -> anyhow::Result<Vec<PathBuf>> {
        unreachable!()
    }
}
