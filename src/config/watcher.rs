use crate::config::{load_configs, Config};
use anyhow::Result;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent};
use nix::sys::select::FdSet;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ConfigWatcher {
    files: Vec<PathBuf>,
    debounce_events: bool,
    timer_fd: RawFd,
    #[allow(warnings)]
    timer: TimerFd,
    inotify: Inotify,
}

impl ConfigWatcher {
    pub fn new(watch: bool, files: Vec<PathBuf>) -> Result<(Option<RawFd>, Option<Inotify>, Option<Self>)> {
        if !watch {
            return Ok((None, None, None));
        }

        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
        for file in &files {
            inotify.add_watch(
                file.parent().expect("config file has a parent directory"),
                AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
            )?;
            inotify.add_watch(file, AddWatchFlags::IN_MODIFY)?;
        }

        let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty())?;

        let this = Self {
            files,
            debounce_events: false,
            timer_fd: timer.as_raw_fd(),
            timer,
            inotify,
        };

        Ok((Some(this.timer_fd), Some(this.inotify), Some(this)))
    }

    pub fn handle(&mut self, readable_fds: FdSet) -> Result<Option<Config>> {
        if readable_fds.contains(self.timer_fd) {
            todo!()
        }

        if let Ok(events) = self.inotify.read_events() {
            if self.config_changed(events)? {
                if self.debounce_events {
                    todo!()
                } else {
                    return Ok(Some(self.get_config()?));
                }
            }
        }

        Ok(None)
    }

    fn get_config(&mut self) -> Result<Config> {
        let result = load_configs(&self.files);
        match &result {
            Ok(_) => {
                println!("Reloading Config");
            }
            Err(_) => {}
        }

        result.map_err(|err| anyhow::format_err!("{err}"))
    }

    fn config_changed(&self, events: Vec<InotifyEvent>) -> Result<bool> {
        //Re-add AddWatchFlags if config file has been deleted then recreated or overwritten by renaming another file to its own name
        for event in &events {
            if event
                .mask
                .intersects(AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO)
            {
                for config_path in &self.files {
                    if config_path.file_name().unwrap_or_default() == event.name.clone().unwrap_or_default() {
                        self.inotify.add_watch(config_path, AddWatchFlags::IN_MODIFY)?;
                    }
                }
            }
        }
        for event in &events {
            match (event.mask, &event.name) {
                // Dir events
                (_, Some(name))
                    if self
                        .files
                        .iter()
                        .any(|p| name == p.file_name().expect("Config path has a file name")) =>
                {
                    return Ok(true)
                }
                // File events
                (mask, _) if mask.contains(AddWatchFlags::IN_MODIFY) => return Ok(true),
                // Unrelated
                _ => (),
            }
        }

        Ok(false)
    }
}
