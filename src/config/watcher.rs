use crate::config::{load_configs, Config};
use crate::main_controller::MainController;
use anyhow::Result;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent};
use nix::sys::select::FdSet;
use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{ClockId, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
pub struct ConfigWatcher {
    files: Vec<PathBuf>,
    debounce: Option<Duration>,
    notifications: bool,
    timer_fd: RawFd,
    timer: TimerFd,
    inotify: Inotify,
    change_pending: bool,
}

impl ConfigWatcher {
    pub fn new(
        watch: bool,
        files: Vec<PathBuf>,
        debounce_ms: u64,
        notifications: bool,
    ) -> Result<(Option<RawFd>, Option<Inotify>, Option<Self>)> {
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

        let debounce = if debounce_ms == 0 {
            None
        } else {
            Some(Duration::from_millis(debounce_ms))
        };

        let this = Self {
            files,
            debounce,
            notifications,
            timer_fd: timer.as_raw_fd(),
            timer,
            inotify,
            change_pending: false,
        };

        Ok((Some(this.timer_fd), Some(this.inotify), Some(this)))
    }

    pub fn handle(&mut self, readable_fds: FdSet, mainctrl: &mut MainController) -> Result<Option<Config>> {
        if readable_fds.contains(self.timer_fd) {
            return Ok(Some(self.get_config(mainctrl)?));
        }

        if let Ok(events) = self.inotify.read_events() {
            if self.config_changed(events)? {
                match self.debounce {
                    Some(debounce) => {
                        // Could already be set, but reset is the debounce.
                        self.change_pending = true;
                        self.timer
                            .set(Expiration::OneShot(TimeSpec::from_duration(debounce)), TimerSetTimeFlags::empty())?;
                    }
                    None => {
                        return Ok(Some(self.get_config(mainctrl)?));
                    }
                };
            }
        }

        Ok(None)
    }

    fn get_config(&mut self, mainctrl: &mut MainController) -> Result<Config> {
        self.change_pending = false;
        self.timer.unset()?;
        let result = load_configs(&self.files);
        match &result {
            Ok(_) => {
                println!("Reloading Config");
            }
            Err(err) => {
                if self.notifications {
                    mainctrl.show_popup("Config error", Some(&err.to_string()));
                }
            }
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
