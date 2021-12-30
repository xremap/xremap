use crate::config::Config;
use crate::device::{device_watcher, input_devices, output_device};
use crate::event_handler::EventHandler;
use evdev::uinput::VirtualDevice;
use evdev::{Device, EventType};
use getopts::Options;
use nix::sys::inotify::Inotify;
use nix::sys::select::select;
use nix::sys::select::FdSet;
use std::env;
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::process::exit;

extern crate getopts;

mod client;
mod command;
mod config;
mod device;
mod event_handler;

fn main() {
    env_logger::init();
    let argv: Vec<String> = env::args().collect();
    let program = argv[0].clone();

    let mut opts = Options::new();
    opts.optmulti("", "device", "Include a device name or path", "NAME");
    opts.optmulti("", "ignore", "Exclude a device name or path", "NAME");
    opts.optflag("", "watch", "Add new devices automatically");
    opts.optflag("h", "help", "print this help menu");

    let args = match opts.parse(&argv[1..]) {
        Ok(args) => args,
        Err(e) => abort(&e.to_string()),
    };
    if args.opt_present("h") {
        println!("{}", &usage(&program, opts));
        return;
    }

    let filename = match &args.free.iter().map(String::as_str).collect::<Vec<&str>>()[..] {
        &[filename] => filename,
        &[..] => abort(&usage(&program, opts)),
    };
    let config = match config::load_config(&filename) {
        Ok(config) => config,
        Err(e) => abort(&format!("Failed to load config '{}': {}", filename, e)),
    };

    let watch = args.opt_present("watch");
    loop {
        let output_device = match output_device() {
            Ok(output_device) => output_device,
            Err(e) => abort(&format!("Failed to prepare an output device: {}", e)),
        };
        let input_devices = match input_devices(&args.opt_strs("device"), &args.opt_strs("ignore"), watch) {
            Ok(input_devices) => input_devices,
            Err(e) => abort(&format!("Failed to prepare input devices: {}", e)),
        };

        if let Err(e) = event_loop(output_device, input_devices, &config, watch) {
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

fn select_readable(devices: &Vec<Device>, watcher: &Option<Inotify>) -> Result<FdSet, Box<dyn Error>> {
    let mut read_fds = FdSet::new();
    for device in devices {
        read_fds.insert(device.as_raw_fd());
    }
    if let Some(inotify) = watcher {
        read_fds.insert(inotify.as_raw_fd());
    }
    select(None, &mut read_fds, None, None, None)?;
    return Ok(read_fds);
}

fn usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} CONFIG [options]", program);
    opts.usage(&brief)
}

fn abort(message: &str) -> ! {
    println!("{}", message);
    exit(1);
}
