extern crate evdev;
extern crate nix;

use crate::output::build_device;
use crate::transform::on_event;
use crate::Config;
use evdev::{Device, EventType};
use nix::sys::select::select;
use nix::sys::select::FdSet;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

pub fn select_device(_devices: &Vec<String>) -> Result<Vec<Device>, Box<dyn Error>> {
    let path_devices = list_devices()?;
    let mut paths: Vec<&String> = path_devices.keys().collect();
    paths.sort_by(|a, b| device_index(a).partial_cmp(&device_index(b)).unwrap());
    println!("Selecting from the following devices:");
    println!("------------------------------------------------------------------------------");
    for path in paths {
        if let Some(device) = path_devices.get(path) {
            println!("{:18}: {}", path, device.name().unwrap_or("<Unnamed device>"));
        }
    }
    println!("------------------------------------------------------------------------------");

    let device = Device::open("/dev/input/event19")?;
    return Ok(vec![device]);
}

pub fn event_loop(mut input_devices: Vec<Device>, _config: &Config) -> Result<(), Box<dyn Error>> {
    let mut input_device = input_devices.remove(0);
    let mut output_device = build_device(&input_device)
        .map_err(|e| format!("Failed to build an output device: {}", e))?;
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

fn device_index(path: &str) -> i32 {
    path.trim_start_matches("/dev/input/event")
        .parse::<i32>()
        .unwrap()
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
