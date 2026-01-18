use crate::client::print_open_windows;
use crate::config::Config;
use crate::device::{
    device_watcher, get_input_devices, output_device, print_device_details, print_device_list, DEVICE_NAME,
};
use crate::event_handler::EventHandler;
use action_dispatcher::ActionDispatcher;
use anyhow::{anyhow, bail, Context};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use client::build_client;
use config::{config_watcher, load_configs};
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
use std::path::PathBuf;
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
#[cfg(test)]
mod tests_disguised_events_in;
#[cfg(test)]
mod tests_disguised_events_out;
#[cfg(test)]
mod tests_extra_modifiers;
#[cfg(test)]
mod tests_keymap_mark;
#[cfg(test)]
mod tests_keymap_mode;
#[cfg(test)]
mod tests_modmap_keys;
#[cfg(test)]
mod tests_modmap_mul_purpose;
#[cfg(test)]
mod tests_modmap_press_release_key;
#[cfg(test)]
mod tests_nested_remap;
#[cfg(test)]
mod tests_virtual_modifier;
mod util;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Limit input devices to the given names or paths. Default is all keyboards.
    #[arg(long, value_delimiter = ',')]
    device: Vec<String>,
    /// Ignore input devices with the given names or paths.
    #[arg(long, value_delimiter = ',')]
    ignore: Vec<String>,
    /// Listen to mouse devices. Default is false.
    #[arg(long)]
    mouse: bool,
    /// Watch for new devices or changing configuration files.
    /// Default is not watching for either.
    /// Examples
    /// - xremap --watch config.yml               # watch devices
    /// - xremap --watch=config config.yml        # watch configuration files
    /// - xremap --watch=config,device config.yml # watch both
    #[arg(long, value_enum, num_args = 0.., value_delimiter = ',', require_equals = true,
           default_missing_value = "device", verbatim_doc_comment)]
    watch: Vec<WatchTargets>,
    /// Generate shell completions
    ///
    /// You can use them by storing in your shells completion file or by running
    /// - in bash: eval "$(xremap --completions bash)"
    /// - in fish: xremap --completions fish | source
    #[arg(long, value_enum, display_order = 100, value_name = "SHELL", verbatim_doc_comment)]
    completions: Option<Shell>,
    /// Choose the name of the created output device.
    /// Default is 'xremap' or 'xremap pid=xx'
    #[arg(long)]
    output_device_name: Option<String>,
    /// Config file(s)
    ///
    /// When more than one file is given, then will modmap, keymap and virtual_modifiers
    /// from the subsequent files be merged into the first configuration file.
    #[arg(required_unless_present = "completions",
        required_unless_present = "list_devices",
        required_unless_present = "device_details",
        required_unless_present = "list_windows",
        num_args = 1..)]
    configs: Vec<PathBuf>,
    /// Choose the vendor value of the created output device.
    /// Must be given in hexadecimal with or without a prefix '0x'.
    /// Default is: 0x1234
    #[arg(long)]
    vendor: Option<String>,
    /// Choose the product value of the created output device.
    /// Must be given in hexadecimal with or without a prefix '0x'.
    /// Default is: 0x5678
    #[arg(long)]
    product: Option<String>,
    /// List info about devices
    #[arg(long)]
    list_devices: bool,
    /// Show device details
    #[arg(long)]
    device_details: bool,
    #[arg(long)]
    list_windows: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum WatchTargets {
    /// add new devices automatically
    Device,
    /// reload the config automatically
    Config,
}

// TODO: Unify this with Event
enum ReloadEvent {
    ReloadConfig,
    ReloadDevices,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args {
        device: device_filter,
        ignore: ignore_filter,
        mouse,
        watch,
        configs,
        completions,
        output_device_name,
        product,
        vendor,
        list_devices,
        device_details,
        list_windows,
    } = Args::parse();

    if let Some(shell) = completions {
        clap_complete::generate(shell, &mut Args::command(), "xremap", &mut stdout());
        return Ok(());
    }

    if device_details {
        print_device_details()?;
        return Ok(());
    }

    if list_devices {
        print_device_list()?;
        return Ok(());
    }

    if list_windows {
        return print_open_windows();
    }

    if let Some(output_device_name) = output_device_name {
        unsafe {
            DEVICE_NAME = Some(output_device_name);
        }
    }

    // Configuration
    let config_paths = match configs[..] {
        [] => panic!("config is set, if not completions"),
        _ => configs,
    };

    let mut config = match config::load_configs(&config_paths) {
        Ok(config) => config,
        Err(e) => bail!(
            "Failed to load config '{}': {}",
            config_paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join("', '"),
            e
        ),
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
    let config_watcher = config_watcher(watch_config, &config_paths).context("Setting up config watcher")?;
    let watchers: Vec<_> = device_watcher.iter().chain(config_watcher.iter()).collect();
    let mut handler = EventHandler::new(timer, &config.default_mode, delay, build_client());
    let vendor = u16::from_str_radix(vendor.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x1234);
    let product = u16::from_str_radix(product.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x5678);
    let output_device = match output_device(
        input_devices.values().next().map(InputDevice::bus_type),
        config.enable_wheel,
        vendor,
        product,
    ) {
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
                        &config_paths,
                        inotify,
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
                if let Ok(c) = load_configs(&config_paths) {
                    println!("Reloading Config");
                    config = c;
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
    let mut device_exists = true;
    let events = match input_device.fetch_events().map_err(|e| (e.raw_os_error(), e)) {
        Err((Some(ENODEV), _)) => {
            device_exists = false;
            Ok(Vec::new())
        }
        Err((_, error)) => Err(error).context("Error fetching input events"),
        Ok(events) => Ok(events.collect()),
    }?;
    let input_events = events.iter().map(|e| Event::new(input_device.to_info(), *e)).collect();
    handle_events(handler, dispatcher, config, input_events)?;
    Ok(device_exists)
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
    let mut run = |command: &Vec<String>| -> anyhow::Result<bool> { handler.delegate_to_client(&command) };
    for action in actions {
        dispatcher.on_action(action, &mut run)?;
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
    config_paths: &Vec<PathBuf>,
    inotify: Inotify,
) -> anyhow::Result<bool> {
    //Re-add AddWatchFlags if config file has been deleted then recreated or overwritten by renaming another file to its own name
    for event in &events {
        if event
            .mask
            .intersects(AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO)
        {
            for config_path in config_paths {
                if config_path.file_name().unwrap_or_default() == event.name.clone().unwrap_or_default() {
                    inotify.add_watch(config_path, AddWatchFlags::IN_MODIFY)?;
                }
            }
        }
    }
    for event in &events {
        match (event.mask, &event.name) {
            // Dir events
            (_, Some(name))
                if config_paths
                    .iter()
                    .any(|p| name == p.file_name().expect("Config path has a file name")) =>
            {
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
