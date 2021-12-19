extern crate evdev;
extern crate nix;

use crate::output::build_device;
use crate::transform::on_event;
use crate::Config;
use evdev::{Device, EventType, Key};
use nix::sys::select::select;
use nix::sys::select::FdSet;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

static UNNAMED_DEVICE: &str = "<Unnamed device>";
static SEPARATOR: &str =
    "------------------------------------------------------------------------------";

pub fn select_device(device_opts: &Vec<String>) -> Result<Vec<Device>, Box<dyn Error>> {
    let mut path_devices = list_devices()?;
    let mut paths: Vec<String> = path_devices.keys().map(|e| e.clone()).collect();
    paths.sort_by(|a, b| device_index(a).partial_cmp(&device_index(b)).unwrap());

    println!("Selecting devices from the following list:");
    println!("{}", SEPARATOR);
    for path in &paths {
        if let Some(device) = path_devices.get(path) {
            print_device(path, device);
        }
    }
    println!("{}", SEPARATOR);

    if device_opts.is_empty() {
        println!("Selected keyboards automatically since --device options weren't specified:");
    } else {
        println!("Selected devices matching {:?}:", device_opts);
    };
    for path in &paths {
        if let Some(device) = path_devices.get(path) {
            let matched = if device_opts.is_empty() {
                is_keyboard(device)
            } else {
                match_device(path, device, device_opts)
            };
            if !matched {
                path_devices.remove(path);
            }
        }
    }

    println!("{}", SEPARATOR);
    for (path, device) in path_devices.iter() {
        print_device(path, device);
    }
    println!("{}", SEPARATOR);

    return Ok(path_devices.into_values().collect());
}

pub fn event_loop(mut input_devices: Vec<Device>, _config: &Config) -> Result<(), Box<dyn Error>> {
    let mut input_device = input_devices.remove(0);
    let mut output_device =
        build_device().map_err(|e| format!("Failed to build an output device: {}", e))?;
    input_device
        .grab()
        .map_err(|e| format!("Failed to grab an input device: {}", e))?;

    loop {
        if !is_readable(&mut input_device)? {
            continue;
        }

        for event in input_device.fetch_events()? {
            if event.event_type() == EventType::KEY {
                on_event(event, &mut output_device)?;
            } else {
                output_device.emit(&[event])?;
            }
        }
    }
}

// We can't know the device path from evdev::enumerate(). So we re-implement it.
fn list_devices() -> Result<HashMap<String, Device>, Box<dyn Error>> {
    let mut path_devices: HashMap<String, Device> = HashMap::new();
    if let Some(dev_input) = read_dir("/dev/input").as_mut().ok() {
        while let Some(entry) = dev_input.next() {
            let path = entry?.path();
            if let Some(fname) = path.file_name() {
                if fname.as_bytes().starts_with(b"event") {
                    let device = Device::open(&path)?;
                    if let Ok(path) = path.into_os_string().into_string() {
                        path_devices.insert(path, device);
                    }
                }
            }
        }
    }
    return Ok(path_devices);
}

fn print_device(path: &str, device: &Device) {
    println!("{:18}: {}", path, device.name().unwrap_or(UNNAMED_DEVICE));
}

fn device_index(path: &str) -> i32 {
    path.trim_start_matches("/dev/input/event")
        .parse::<i32>()
        .unwrap()
}

fn match_device(path: &str, device: &Device, device_opts: &Vec<String>) -> bool {
    for device_opt in device_opts {
        // Check exact matches for explicit selection
        if path == device_opt || device.name().unwrap_or(UNNAMED_DEVICE) == device_opt {
            return true;
        }
        // eventXX shorthand for /dev/input/eventXX
        if device_opt.starts_with("event") && path == format!("/dev/input/{}", device_opt) {
            return true;
        }
        // Allow partial matches for device names
        if device.name().unwrap_or(UNNAMED_DEVICE).contains(device_opt) {
            return true;
        }
    }
    return false;
}

fn is_keyboard(device: &Device) -> bool {
    // Credit: https://github.com/mooz/xkeysnail/blob/master/xkeysnail/input.py#L17-L32
    match device.supported_keys() {
        Some(keys) => {
            keys.contains(Key::KEY_SPACE)
                && keys.contains(Key::KEY_A)
                && keys.contains(Key::KEY_Z)
                && !keys.contains(Key::BTN_LEFT) // BTN_MOUSE
        }
        None => false,
    }
}

fn is_readable(device: &mut Device) -> Result<bool, Box<dyn Error>> {
    let mut read_fds = FdSet::new();
    read_fds.insert(device.as_raw_fd());
    select(None, &mut read_fds, None, None, None)?;
    for fd in read_fds.fds(None) {
        if fd == device.as_raw_fd() {
            return Ok(true);
        }
    }
    return Ok(false);
}
