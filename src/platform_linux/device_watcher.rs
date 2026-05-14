use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::os::fd::{AsFd, BorrowedFd};
use std::path::PathBuf;

#[derive(Debug)]
pub struct DeviceWatcher {
    inotify: Inotify,
}

impl AsFd for DeviceWatcher {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inotify.as_fd()
    }
}

impl DeviceWatcher {
    pub fn new(watch: bool) -> anyhow::Result<Option<Self>> {
        if watch {
            let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
            inotify.add_watch("/dev/input", AddWatchFlags::IN_CREATE | AddWatchFlags::IN_ATTRIB)?;
            Ok(Some(Self { inotify }))
        } else {
            Ok(None)
        }
    }

    pub fn read_events(&self) -> anyhow::Result<Vec<PathBuf>> {
        Ok(self
            .inotify
            .read_events()?
            .into_iter()
            .filter_map(|event| Some(PathBuf::from("/dev/input/").join(event.name?)))
            .collect())
    }
}
