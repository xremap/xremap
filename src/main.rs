use crate::config::Config;
use crate::device::{device_watcher, input_devices, output_device};
use crate::event_handler::EventHandler;
use clap::{ArgEnum, Parser};
use evdev::uinput::VirtualDevice;
use evdev::{Device, EventType};
use nix::sys::inotify::Inotify;
use nix::sys::select::select;
use nix::sys::select::FdSet;
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process::exit;

mod client;
mod config;
mod device;
mod event_handler;

#[derive(Parser, Debug)]
#[clap(version)]
struct Opts {
    /// Include a device name or path
    #[clap(long, use_delimiter = true)]
    device: Vec<String>,
    /// Ignore a device name or path
    #[clap(long, use_delimiter = true)]
    ignore: Vec<String>,
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
    /// Targets to watch
    ///
    /// - Device to add new devices automatically
    /// - Config to reload the config automatically
    watch: Vec<WatchTargets>,
    /// Config file
    config: PathBuf,
}

#[derive(ArgEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum WatchTargets {
    /// Device to add new devices automatically
    Device,
    /// Config to reload the config automatically
    Config,
}

fn main() {
    env_logger::init();

    let Opts {
        device,
        ignore,
        watch,
        config,
    } = dbg!(Opts::parse());

    let config = match config::load_config(&config) {
        Ok(config) => config,
        Err(e) => abort(&format!("Failed to load config '{}': {}", config.display(), e)),
    };

    let watch_devices = watch.contains(&WatchTargets::Device);

    loop {
        let output_device = match output_device() {
            Ok(output_device) => output_device,
            Err(e) => abort(&format!("Failed to prepare an output device: {}", e)),
        };
        let input_devices = match input_devices(&device, &ignore, watch_devices) {
            Ok(input_devices) => input_devices,
            Err(e) => abort(&format!("Failed to prepare input devices: {}", e)),
        };

        if let Err(e) = event_loop(output_device, input_devices, &config, watch_devices) {
            if e.to_string().starts_with("No such device") {
                println!("Found a removed device. Reselecting devices.");
                continue;
            }
            abort(&format!("Error: {}", e));
        }
    }
}

fn event_loop(
    output_device: VirtualDevice,
    mut input_devices: Vec<Device>,
    config: &Config,
    watch: bool,
) -> Result<(), Box<dyn Error>> {
    let watcher = device_watcher(watch)?;
    let mut handler = EventHandler::new(output_device);
    loop {
        let readable_fds = select_readable(&input_devices, &watcher)?;
        for input_device in &mut input_devices {
            if readable_fds.contains(input_device.as_raw_fd()) {
                for event in input_device.fetch_events()? {
                    if event.event_type() == EventType::KEY {
                        handler.on_event(event, config)?;
                    } else {
                        handler.send_event(event)?;
                    }
                }
            }
        }
        if let Some(inotify) = watcher {
            if readable_fds.contains(inotify.as_raw_fd()) {
                println!("Detected device changes. Reselecting devices.");
                return Ok(());
            }
        }
    }
}

fn select_readable(devices: &[Device], watcher: &Option<Inotify>) -> Result<FdSet, Box<dyn Error>> {
    let mut read_fds = FdSet::new();
    for device in devices {
        read_fds.insert(device.as_raw_fd());
    }
    if let Some(inotify) = watcher {
        read_fds.insert(inotify.as_raw_fd());
    }
    select(None, &mut read_fds, None, None, None)?;
    Ok(read_fds)
}

fn abort(message: &str) -> ! {
    println!("{}", message);
    exit(1);
}
