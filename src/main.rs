use crate::client::print_open_windows;
use crate::config::{Config, ConfigWatcher};
use crate::device::{
    device_watcher, open_device, output_device, print_device_details, print_device_list, select_input_devices,
    DEVICE_NAME,
};
use crate::event_handler::EventHandler;
use crate::main_controller::MainController;
use crate::operator_handler::OperatorHandler;
use crate::operators::get_operator_handler;
use crate::throttle_emit::ThrottleEmit;
use crate::timeout_manager::TimeoutManager;
use action_dispatcher::ActionDispatcher;
use anyhow::{anyhow, bail, Context};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use device::InputDevice;
use event::Event;
use nix::libc::ENODEV;
use nix::sys::inotify::{Inotify, InotifyEvent};
use nix::sys::select::select;
use nix::sys::select::FdSet;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::collections::HashMap;
use std::io::stdout;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

mod action;
mod action_dispatcher;
mod bridge;
mod client;
mod command_runner;
mod config;
mod device;
mod emit_handler;
mod event;
mod event_handler;
mod main_controller;
mod operator_double_tap;
mod operator_handler;
mod operator_sim;
mod operators;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_disguised_events_in;
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
mod tests_modmap_mul_purpose_tap_preferred;
#[cfg(test)]
mod tests_modmap_press_release_key;
#[cfg(test)]
mod tests_nested_remap;
#[cfg(test)]
mod tests_operator_double_tap;
#[cfg(test)]
mod tests_operator_handler;
#[cfg(test)]
mod tests_operator_sim;
#[cfg(test)]
mod tests_throttle_emit;
#[cfg(test)]
mod tests_virtual_modifier;
mod throttle_emit;
mod timeout_manager;
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
        required_unless_present = "bridge",
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
    /// List open windows. Use this to get app_class and title. It only works for COSMIC. Since v0.14.10.
    #[arg(long)]
    list_windows: bool,
    /// Suppress logging of window title and application changes. Default is false. Since v0.14.10.
    #[arg(long)]
    no_window_logging: bool,
    /// Allow remappings to execute programs. Default is ambiguous. Since v0.15.1
    #[arg(long)]
    allow_launch: Option<bool>,
    /// Open a bridge from the desktop environment to the xremap system service. Since v0.15.1
    #[arg(long)]
    bridge: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum WatchTargets {
    /// add new devices automatically
    Device,
    /// reload the config automatically
    Config,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args {
        device: device_filter,
        ignore: ignore_filter,
        mouse,
        watch,
        configs: config_paths,
        completions,
        output_device_name,
        product,
        vendor,
        list_devices,
        device_details,
        list_windows,
        no_window_logging,
        allow_launch,
        bridge,
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

    if bridge {
        // Default deny launch
        return bridge::main(!no_window_logging, allow_launch.unwrap_or(false));
    }

    if let Some(output_device_name) = output_device_name {
        unsafe {
            DEVICE_NAME = Some(output_device_name);
        }
    }

    // Configuration
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

    let timeout_manager = Rc::new(TimeoutManager::new());
    let timeout_manager_fd = timeout_manager.get_timer_fd();

    // Device name

    let own_device: &str = InputDevice::current_name();

    // Event listeners
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty())?;
    let timer_fd = timer.as_raw_fd();
    let delay = Duration::from_millis(config.keypress_delay_ms);
    let mut input_devices = select_input_devices(&device_filter, &ignore_filter, mouse, watch_devices, &own_device)?;
    let device_watcher = device_watcher(watch_devices).context("Setting up device watcher")?;
    let (config_watcher_fd, config_watcher_inotify, mut config_watcher) =
        ConfigWatcher::new(watch_config, config_paths, config.config_watch_debounce_ms, config.notifications)?;
    let watchers: Vec<_> = device_watcher.iter().chain(config_watcher_inotify.iter()).collect();

    // wmclient
    // Default allow launch (Change to false in a major upgrade)
    let mut mainctrl = MainController::new(!no_window_logging, allow_launch.unwrap_or(true));

    // EventHandler
    let mut handler = EventHandler::new(timer, &config.default_mode, delay);
    let vendor = u16::from_str_radix(vendor.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x1234);
    let product = u16::from_str_radix(product.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x5678);
    let output_device = match output_device(
        input_devices.values().next().map(InputDevice::bus_type),
        config.enable_wheel,
        vendor,
        product,
        &own_device,
    ) {
        Ok(output_device) => output_device,
        Err(e) => bail!("Failed to prepare an output device: {}", e),
    };

    let throttle_emit = if config.throttle_ms == 0 {
        None
    } else {
        Some(ThrottleEmit::new(Duration::from_millis(config.throttle_ms)))
    };

    let mut operator_handler = get_operator_handler(&config, timeout_manager.clone());

    let mut dispatcher = ActionDispatcher::new(output_device, throttle_emit);

    // Main loop
    loop {
        if config.notifications {
            mainctrl.show_popup("Ready", None);
        }

        'event_loop: loop {
            let readable_fds =
                select_readable(input_devices.values(), &watchers, timer_fd, timeout_manager_fd, config_watcher_fd)?;
            if readable_fds.contains(timer_fd) {
                if let Err(error) = handle_events(
                    &mut handler,
                    &mut dispatcher,
                    &config,
                    vec![Event::OverrideTimeout],
                    &mut operator_handler,
                    &mut mainctrl,
                ) {
                    println!("Error on remap timeout: {error}")
                }
            }

            if readable_fds.contains(timeout_manager_fd) {
                if timeout_manager.need_timeout()? {
                    if let Err(error) = handle_events(
                        &mut handler,
                        &mut dispatcher,
                        &mut config,
                        vec![Event::Tick],
                        &mut operator_handler,
                        &mut mainctrl,
                    ) {
                        println!("Error on timeout: {error}")
                    }
                }
            }

            for input_device in input_devices.values_mut() {
                if !readable_fds.contains(input_device.as_raw_fd()) {
                    continue;
                }

                if !handle_input_events(
                    input_device,
                    &mut handler,
                    &mut dispatcher,
                    &config,
                    &mut operator_handler,
                    &mut mainctrl,
                )? {
                    println!("Found a removed device. Reselecting devices.");

                    for input_device in input_devices.values_mut() {
                        input_device.ungrab();
                    }

                    input_devices =
                        select_input_devices(&device_filter, &ignore_filter, mouse, watch_devices, &own_device)?;

                    continue 'event_loop;
                }
            }

            if let Some(inotify) = device_watcher {
                if let Ok(events) = inotify.read_events() {
                    handle_device_changes(
                        events,
                        &mut input_devices,
                        &device_filter,
                        &ignore_filter,
                        mouse,
                        &own_device,
                    )?;
                }
            }

            if let Some(config_watcher) = config_watcher.as_mut() {
                match config_watcher.handle(readable_fds, &mut mainctrl) {
                    Ok(Some(c)) => {
                        config = c;
                        break 'event_loop;
                    }
                    _ => {
                        continue 'event_loop;
                    }
                };
            }
        }
    }
}

fn select_readable<'a>(
    devices: impl Iterator<Item = &'a InputDevice>,
    watchers: &[&Inotify],
    timer_fd: RawFd,
    timeout_manager_fd: RawFd,
    config_watcher_fd: Option<RawFd>,
) -> anyhow::Result<FdSet> {
    let mut read_fds = FdSet::new();
    read_fds.insert(timer_fd);
    read_fds.insert(timeout_manager_fd);
    config_watcher_fd.map(|fd| read_fds.insert(fd));
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
    config: &Config,
    operator_handler: &mut Option<OperatorHandler>,
    mainctrl: &mut MainController,
) -> anyhow::Result<bool> {
    let events: Vec<_> = match input_device.fetch_events() {
        Err(err) if err.raw_os_error() == Some(ENODEV) => {
            // The device doesn't exist anymore.
            return Ok(false);
        }
        events => events.context("Error fetching input events")?,
    }
    .collect();

    let info = Rc::new(input_device.to_info());
    let input_events = events.iter().map(|e| Event::new(info.clone(), *e)).collect();
    handle_events(handler, dispatcher, config, input_events, operator_handler, mainctrl)?;
    Ok(true)
}

// Handle an Event with EventHandler, and dispatch Actions with ActionDispatcher
fn handle_events(
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &Config,
    mut events: Vec<Event>,
    operator_handler: &mut Option<OperatorHandler>,
    mainctrl: &mut MainController,
) -> anyhow::Result<()> {
    if let Some(handler) = operator_handler {
        events = handler.map_events(events);
    };
    let actions = handler
        .on_events(&events, config, mainctrl.wmclient())
        .map_err(|e| anyhow!("Failed handling {events:?}:\n  {e:?}"))?;
    for action in actions {
        dispatcher.on_action(action, mainctrl)?;
    }
    Ok(())
}

fn handle_device_changes(
    events: Vec<InotifyEvent>,
    input_devices: &mut HashMap<PathBuf, InputDevice>,
    device_filter: &[String],
    ignore_filter: &[String],
    mouse: bool,
    own_device: &str,
) -> anyhow::Result<()> {
    input_devices.extend(events.into_iter().filter_map(|event| {
        event.name.and_then(|name| {
            let path = PathBuf::from("/dev/input/").join(name);
            let mut device = open_device(path)?;
            if device.is_input_device(device_filter, ignore_filter, mouse, own_device) && device.grab() {
                device.print();
                Some(device.into())
            } else {
                None
            }
        })
    }));
    Ok(())
}
