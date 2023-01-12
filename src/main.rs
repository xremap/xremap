use crate::config::Config;
use crate::device::{device_watcher, get_input_devices, output_device};
use crate::event_handler::EventHandler;
use action_dispatcher::ActionDispatcher;
use anyhow::{anyhow, bail, Context};
use clap::{AppSettings, ArgEnum, IntoApp, Parser};
use clap_complete::Shell;
use client::build_client;
use config::{config_watcher, load_config};
use device::InputDevice;
use event::Event;
use nix::libc::ENODEV;
use nix::sys::inotify::{AddWatchFlags, Inotify, InotifyEvent};
use nix::sys::select::select;
use nix::sys::select::FdSet;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::collections::HashMap;
use std::io::stdout;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::time::Duration;

mod action;
mod action_dispatcher;
mod client;
mod config;
mod device;
mod event;
mod event_handler;
#[cfg(test)]
mod tests;

#[derive(Parser, Debug)]
#[clap(version, global_setting(AppSettings::DeriveDisplayOrder))]
struct Opts {
    /// Include a device name or path
    #[clap(long, use_delimiter = true)]
    device: Vec<String>,
    /// Ignore a device name or path
    #[clap(long, use_delimiter = true)]
    ignore: Vec<String>,
    /// Match mice by default
    #[clap(long)]
    mouse: bool,
    /// Targets to watch
    ///
    /// - device: add new devices automatically
    /// - config: reload the config automatically
    #[clap(
        long,
        arg_enum,
        min_values = 0,
        use_delimiter = true,
        require_equals = true,
        default_missing_value = "device",
        verbatim_doc_comment,
        hide_possible_values = true,
        // Separating the help like this is necessary due to
        // https://github.com/clap-rs/clap/issues/3312
        help = "Targets to watch [possible values: device, config]"
    )]
    watch: Vec<WatchTargets>,
    /// Generate shell completions
    ///
    /// You can use them by storing in your shells completion file or by running
    /// - in bash: eval "$(xremap --completions bash)"
    /// - in fish: xremap --completions fish | source
    #[clap(long, arg_enum, display_order = 100, value_name = "SHELL", verbatim_doc_comment)]
    completions: Option<Shell>,
    /// Config file
    #[clap(required_unless_present = "completions")]
    config: Option<PathBuf>,
}

#[derive(ArgEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum WatchTargets {
    /// Device to add new devices automatically
    Device,
    /// Config to reload the config automatically
    Config,
}

// TODO: Unify this with Event
enum ReloadEvent {
    ReloadConfig,
    ReloadDevices,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Opts {
        device: device_filter,
        ignore: ignore_filter,
        mouse,
        watch,
        config,
        completions,
    } = Opts::parse();

    if let Some(shell) = completions {
        clap_complete::generate(shell, &mut Opts::into_app(), "xremap", &mut stdout());
        return Ok(());
    }

    // Configuration
    let config_path = config.expect("config is set, if not completions");
    let mut config = match config::load_config(&config_path) {
        Ok(config) => config,
        Err(e) => bail!("Failed to load config '{}': {}", config_path.display(), e),
    };
    let watch_devices = watch.contains(&WatchTargets::Device);
    let watch_config = watch.contains(&WatchTargets::Config);

    // Event listeners
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty())?;
    let timer_fd = timer.as_raw_fd();
    let delay = Duration::from_millis(config.keypress_delay_ms);
    let mut input_devices = match get_input_devices(&device_filter, &ignore_filter, mouse, watch_devices) {
        Ok(input_devices) => input_devices,
        Err(e) => bail!("Failed to prepare input devices: {}", e),
    };
    let device_watcher = device_watcher(watch_devices).context("Setting up device watcher")?;
    let config_watcher = config_watcher(watch_config, &config_path).context("Setting up config watcher")?;
    let watchers: Vec<_> = device_watcher.iter().chain(config_watcher.iter()).collect();
    let mut handler = EventHandler::new(timer, &config.default_mode, delay, build_client());
    let output_device = match output_device(input_devices.values().next().map(InputDevice::bus_type)) {
        Ok(output_device) => output_device,
        Err(e) => bail!("Failed to prepare an output device: {}", e),
    };
    let mut dispatcher = ActionDispatcher::new(output_device);

    // Main loop
    loop {
        match 'event_loop: loop {
            let readable_fds = select_readable(input_devices.values(), &watchers, timer_fd)?;
            if readable_fds.contains(timer_fd) {
                if let Err(error) =
                    handle_events(&mut handler, &mut dispatcher, &mut config, vec![Event::OverrideTimeout])
                {
                    println!("Error on remap timeout: {error}")
                }
            }

            for input_device in input_devices.values_mut() {
                if !readable_fds.contains(input_device.as_raw_fd()) {
                    continue;
                }

                if !handle_input_events(input_device, &mut handler, &mut dispatcher, &mut config)? {
                    println!("Found a removed device. Reselecting devices.");
                    break 'event_loop ReloadEvent::ReloadDevices;
                }
            }

            if let Some(inotify) = device_watcher {
                if let Ok(events) = inotify.read_events() {
                    handle_device_changes(events, &mut input_devices, &device_filter, &ignore_filter, mouse)?;
                }
            }
            if let Some(inotify) = config_watcher {
                if let Ok(events) = inotify.read_events() {
                    if !handle_config_changes(
                        events,
                        &mut input_devices,
                        &device_filter,
                        &ignore_filter,
                        mouse,
                        &config_path,
                    )? {
                        break 'event_loop ReloadEvent::ReloadConfig;
                    }
                }
            }
        } {
            ReloadEvent::ReloadDevices => {
                for input_device in input_devices.values_mut() {
                    input_device.ungrab();
                }
                input_devices = match get_input_devices(&device_filter, &ignore_filter, mouse, watch_devices) {
                    Ok(input_devices) => input_devices,
                    Err(e) => bail!("Failed to prepare input devices: {}", e),
                };
            }
            ReloadEvent::ReloadConfig => {
                match (config.modify_time, config_path.metadata().and_then(|m| m.modified())) {
                    (Some(last_mtime), Ok(current_mtim)) if last_mtime == current_mtim => continue,
                    _ => {
                        if let Ok(c) = load_config(&config_path) {
                            println!("Reloading Config");
                            config = c;
                        }
                    }
                }
            }
        }
    }
}

fn select_readable<'a>(
    devices: impl Iterator<Item = &'a InputDevice>,
    watchers: &[&Inotify],
    timer_fd: RawFd,
) -> anyhow::Result<FdSet> {
    let mut read_fds = FdSet::new();
    read_fds.insert(timer_fd);
    for device in devices {
        read_fds.insert(device.as_raw_fd());
    }
    for inotify in watchers {
        read_fds.insert(inotify.as_raw_fd());
    }
    select(None, &mut read_fds, None, None, None)?;
    Ok(read_fds)
}

// Return false when a removed device is found.
fn handle_input_events(
    input_device: &mut InputDevice,
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &mut Config,
) -> anyhow::Result<bool> {
    match input_device.fetch_events().map_err(|e| (e.raw_os_error(), e)) {
        Err((Some(ENODEV), _)) => Ok(false),
        Err((_, error)) => Err(error).context("Error fetching input events"),
        Ok(events) => {
            let mut input_events: Vec<Event> = Vec::new();
            for event in events {
                let event = Event::new(event);
                input_events.push(event);
            }
            handle_events(handler, dispatcher, config, input_events)?;

            Ok(true)
        }
    }
}

// Handle an Event with EventHandler, and dispatch Actions with ActionDispatcher
fn handle_events(
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &mut Config,
    events: Vec<Event>,
) -> anyhow::Result<()> {
    let actions = handler
        .on_events(&events, config)
        .map_err(|e| anyhow!("Failed handling {events:?}:\n  {e:?}"))?;
    for action in actions {
        dispatcher.on_action(action)?;
    }
    Ok(())
}

fn handle_device_changes(
    events: Vec<InotifyEvent>,
    input_devices: &mut HashMap<PathBuf, InputDevice>,
    device_filter: &[String],
    ignore_filter: &[String],
    mouse: bool,
) -> anyhow::Result<()> {
    input_devices.extend(events.into_iter().filter_map(|event| {
        event.name.and_then(|name| {
            let path = PathBuf::from("/dev/input/").join(name);
            let mut device = InputDevice::try_from(path).ok()?;
            if device.is_input_device(device_filter, ignore_filter, mouse) && device.grab() {
                device.print();
                Some(device.into())
            } else {
                None
            }
        })
    }));
    Ok(())
}

fn handle_config_changes(
    events: Vec<InotifyEvent>,
    input_devices: &mut HashMap<PathBuf, InputDevice>,
    device_filter: &[String],
    ignore_filter: &[String],
    mouse: bool,
    config_path: &Path,
) -> anyhow::Result<bool> {
    for event in &events {
        match (event.mask, &event.name) {
            // Dir events
            (_, Some(name)) if name == config_path.file_name().expect("Config path has a file name") => {
                return Ok(false)
            }
            // File events
            (mask, _) if mask.contains(AddWatchFlags::IN_MODIFY) => return Ok(false),
            // Unrelated
            _ => (),
        }
    }
    input_devices.extend(events.into_iter().filter_map(|event| {
        event.name.and_then(|name| {
            let path = PathBuf::from("/dev/input/").join(name);
            let mut device = InputDevice::try_from(path).ok()?;
            if device.is_input_device(device_filter, ignore_filter, mouse) && device.grab() {
                device.print();
                Some(device.into())
            } else {
                None
            }
        })
    }));
    Ok(true)
}
