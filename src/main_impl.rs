#[cfg(target_os = "freebsd")]
use crate::platform_freebsd::{ConfigWatcher, DeviceWatcher};
#[cfg(target_os = "linux")]
use crate::platform_linux::{ConfigWatcher, DeviceWatcher};

use crate::action_dispatcher::ActionDispatcher;
use crate::client::print_open_windows;
use crate::config::{load_configs, Config};
use crate::device::{
    choose_device_name, open_device, output_device, print_device_details, print_device_list, select_input_devices,
    InputDevice, InputDeviceInfo,
};
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::main_controller::MainController;
use crate::operator_handler::OperatorHandler;
use crate::plugin::{apply_plugin, Plugin};
use crate::throttle_emit::ThrottleEmit;
use crate::timeout_manager::TimeoutManager;
use anyhow::{anyhow, bail, Context};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use nix::libc::ENODEV;
use nix::sys::select::{select, FdSet};
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::collections::HashMap;
use std::io::stdout;
use std::os::fd::{AsFd, RawFd};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

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
    /// Trackpads, tablets and other absolute devices are not supported.
    #[arg(long, verbatim_doc_comment)]
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
        num_args = 1.., verbatim_doc_comment)]
    configs: Vec<PathBuf>,
    /// Choose the vendor value of the created output device.
    /// Must be given in hexadecimal with or without a prefix '0x'.
    /// Default is: 0x1234
    #[arg(long, verbatim_doc_comment)]
    vendor: Option<String>,
    /// Choose the product value of the created output device.
    /// Must be given in hexadecimal with or without a prefix '0x'.
    /// Default is: 0x5678
    #[arg(long, verbatim_doc_comment)]
    product: Option<String>,
    /// List info about devices
    #[arg(long)]
    list_devices: bool,
    /// Show device details
    #[arg(long)]
    device_details: bool,
    /// List open windows. Use this to get app_class and title.
    /// Since v0.15.5. Not supported for GNOME Wayland or KDE Wayland.
    #[arg(long, verbatim_doc_comment)]
    list_windows: bool,
    /// Suppress logging of window title and application changes.
    /// Default is false. Since v0.14.10.
    #[arg(long, verbatim_doc_comment)]
    no_window_logging: bool,
    /// Allow remappings to execute programs. Default is ambiguous. Since v0.15.1
    #[arg(long)]
    allow_launch: Option<bool>,
    /// Open a bridge from the desktop environment to the xremap system service.
    /// Since v0.15.1
    #[arg(long, verbatim_doc_comment)]
    bridge: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum WatchTargets {
    /// add new devices automatically
    Device,
    /// reload the config automatically
    Config,
}

// Action that the main loop must perform.
#[derive(Debug)]
pub enum MainAction {
    #[allow(unused)]
    Exit,
    ReloadConfig,
    RemoveDevice(Rc<InputDeviceInfo>),
}

/// Run xremap CLI
///  - Inits logging.
///  - Parses command-line arguments.
///  - Loads configuration file.
///  - Selects input devices.
///  - Creates output device.
///  - Enters infinite event-loop that listens on input devices.
pub fn xremap_cli(mut plugin: impl Plugin) -> anyhow::Result<()> {
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
        return crate::bridge::main(!no_window_logging, allow_launch.unwrap_or(false));
    }

    // Configuration
    let mut config = match crate::config::load_configs(&config_paths) {
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

    // Device name
    let own_device: String = output_device_name.unwrap_or_else(choose_device_name);

    // Event listeners
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty())?;
    let delay = Duration::from_millis(config.keypress_delay_ms);
    let mut input_devices = select_input_devices(&device_filter, &ignore_filter, mouse, watch_devices, &own_device)?;
    let device_watcher = DeviceWatcher::new(watch_devices).context("Setting up device watcher")?;
    let mut config_watcher = ConfigWatcher::new(watch_config, config_paths.clone(), config.config_watch_debounce_ms)?;

    // wmclient
    // Default allow launch (Change to false in a major upgrade)
    let mut mainctrl = MainController::new(!no_window_logging, allow_launch.unwrap_or(true));

    // OperatorHandler
    let operator_handler = if config.experimental_map.len() > 0 {
        Some(OperatorHandler::new(&config.experimental_map, timeout_manager.clone()))
    } else {
        None
    };

    // EventHandler
    let mut handler = EventHandler::new(timer, &config.default_mode, delay, operator_handler);
    let vendor = u16::from_str_radix(vendor.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x1234);
    let product = u16::from_str_radix(product.unwrap_or_default().trim_start_matches("0x"), 16).unwrap_or(0x5678);
    let output_device = output_device(
        input_devices.values().next().map(InputDevice::bus_type),
        config.enable_wheel,
        vendor,
        product,
        &own_device,
    )
    .context("Failed to prepare an output device")?;

    let throttle_emit = if config.throttle_ms == 0 {
        None
    } else {
        Some(ThrottleEmit::new(Duration::from_millis(config.throttle_ms)))
    };

    let mut dispatcher = ActionDispatcher::new(output_device, throttle_emit);

    if config.notifications {
        mainctrl.show_popup("Ready", None);
    }

    // Main loop
    loop {
        let main_action = event_loop(
            &mut input_devices,
            &device_watcher,
            &mut config_watcher,
            &timeout_manager,
            &mut handler,
            &mut dispatcher,
            &mut config,
            &mut mainctrl,
            &device_filter,
            &ignore_filter,
            mouse,
            &own_device,
            &mut plugin,
        )?;

        match main_action {
            MainAction::Exit => {
                return Ok(());
            }
            MainAction::ReloadConfig => match load_configs(&config_paths) {
                Ok(c) => {
                    println!("Reloading Config");
                    // The new config is only partially used.
                    config = c;
                    if config.notifications {
                        mainctrl.show_popup("Ready", None);
                    }
                }
                Err(err) => {
                    if config.notifications {
                        mainctrl.show_popup("Config error", Some(&err.to_string()));
                    }
                }
            },
            MainAction::RemoveDevice(device_info) => {
                println!("Found a removed device: {:?}", device_info.name);
                input_devices.retain(|path, _| device_info.path != *path);

                if input_devices.is_empty() {
                    if watch_devices {
                        println!("No device was selected, but --watch is waiting for new devices.");
                    } else {
                        bail!("Last device was removed, and not watching for new devices");
                    }
                }
            }
        }
    }
}

fn event_loop(
    input_devices: &mut HashMap<PathBuf, InputDevice>,
    device_watcher: &Option<DeviceWatcher>,
    config_watcher: &mut Option<ConfigWatcher>,
    timeout_manager: &Rc<TimeoutManager>,
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &mut Config,
    mainctrl: &mut MainController,
    device_filter: &[String],
    ignore_filter: &[String],
    mouse: bool,
    own_device: &str,
    plugin: &mut impl Plugin,
) -> anyhow::Result<MainAction> {
    'event_loop: loop {
        let readable_fds =
            select_readable(input_devices.values(), &device_watcher, &config_watcher, &handler, &timeout_manager)?;

        if readable_fds.contains(&handler.as_fd().as_raw_fd()) {
            if let Err(error) =
                handle_events(handler, dispatcher, &config, vec![Event::OverrideTimeout], mainctrl, plugin)
            {
                println!("Error on remap timeout: {error}")
            }
        }

        if readable_fds.contains(&timeout_manager.as_fd().as_raw_fd()) {
            if timeout_manager.need_timeout()? {
                if let Err(error) = handle_events(handler, dispatcher, config, vec![Event::Tick], mainctrl, plugin) {
                    println!("Error on timeout: {error}")
                }
            }
        }

        for input_device in input_devices.values_mut() {
            if !readable_fds.contains(&input_device.as_fd().as_raw_fd()) {
                continue;
            }

            if let Some(main_action) =
                handle_input_events(input_device, handler, dispatcher, &config, mainctrl, plugin)?
            {
                return Ok(main_action);
            }
        }

        if let Some(device_watcher) = &device_watcher {
            if let Ok(events) = device_watcher.read_events() {
                handle_device_changes(events, input_devices, &device_filter, &ignore_filter, mouse, &own_device);
            }
        }

        if let Some(config_watcher) = config_watcher.as_mut() {
            match config_watcher.handle(readable_fds) {
                Ok(Some(action)) => return Ok(action),
                _ => {}
            };
            continue 'event_loop;
        }
    }
}

fn select_readable<'a>(
    devices: impl Iterator<Item = &'a InputDevice>,
    device_watcher: &Option<DeviceWatcher>,
    config_watcher: &Option<ConfigWatcher>,
    event_handler: &impl AsFd,
    timeout_manager: &Rc<TimeoutManager>,
) -> anyhow::Result<Vec<RawFd>> {
    let mut read_fds = FdSet::new();
    read_fds.insert(event_handler.as_fd());
    read_fds.insert(timeout_manager.as_fd());
    for device in devices {
        read_fds.insert(device.as_fd());
    }
    #[cfg(target_os = "linux")]
    if let Some(device_watcher) = device_watcher {
        read_fds.insert(device_watcher.as_fd());
    }
    #[cfg(target_os = "linux")]
    if let Some(config_watcher) = config_watcher {
        read_fds.insert(config_watcher.borrow_timer());
        read_fds.insert(config_watcher.borrow_inotify());
    }
    select(None, &mut read_fds, None, None, None)?;

    // Make the result independent of borrowed fds
    Ok(read_fds.fds(None).map(|fd| fd.as_raw_fd()).collect())
}

fn handle_input_events(
    input_device: &mut InputDevice,
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &Config,
    mainctrl: &mut MainController,
    plugin: &mut impl Plugin,
) -> anyhow::Result<Option<MainAction>> {
    let info = Rc::new(input_device.to_info());
    let events = match input_device.fetch_events() {
        Err(err) if err.raw_os_error() == Some(ENODEV) => {
            // The device doesn't exist anymore.
            return Ok(Some(MainAction::RemoveDevice(info)));
        }
        events => events.context("Error fetching input events")?,
    };

    let input_events = events.map(|e| Event::new(info.clone(), e)).collect();
    handle_events(handler, dispatcher, config, input_events, mainctrl, plugin)?;
    Ok(None)
}

// Handle an Event with EventHandler, and dispatch Actions with ActionDispatcher
fn handle_events<T: Plugin>(
    handler: &mut EventHandler,
    dispatcher: &mut ActionDispatcher,
    config: &Config,
    mut events: Vec<Event>,
    mainctrl: &mut MainController,
    plugin: &mut T,
) -> anyhow::Result<()> {
    if T::IMPLEMENTED {
        events = apply_plugin(plugin, events)
    };

    let actions = handler
        .on_events(events, config, mainctrl.wmclient())
        .map_err(|err| anyhow!("EventHandler failed: {err:?}"))?;
    for action in actions {
        dispatcher.on_action(action, mainctrl)?;
    }
    Ok(())
}

fn handle_device_changes(
    events: Vec<PathBuf>,
    input_devices: &mut HashMap<PathBuf, InputDevice>,
    device_filter: &[String],
    ignore_filter: &[String],
    mouse: bool,
    own_device: &str,
) {
    // Ignore already grabbed devices.
    // A problem that could occur is an old device in `input_devices` which is stale.
    // So ignoring an event for that path would be incorrect. But `handle_input_events` removes
    // the devices reliably, before this function gets an event for a new devive on the same path.
    let mut ignore: Vec<PathBuf> = input_devices.iter().map(|(path, _)| path).cloned().collect();

    input_devices.extend(events.into_iter().filter_map(|path| {
        if ignore.contains(&path) {
            return None;
        }
        ignore.push(path.clone());
        let mut device = open_device(path)?;
        if device.is_input_device(device_filter, ignore_filter, mouse, own_device) && device.grab() {
            device.print();
            Some(device.into())
        } else {
            None
        }
    }));
}
