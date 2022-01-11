extern crate evdev;
extern crate nix;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, Key, RelativeAxisType};
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::process;

static MOUSE_BTNS: [&str; 20] = [
    "BTN_MISC",
    "BTN_0",
    "BTN_1",
    "BTN_2",
    "BTN_3",
    "BTN_4",
    "BTN_5",
    "BTN_6",
    "BTN_7",
    "BTN_8",
    "BTN_9",
    "BTN_MOUSE",
    "BTN_LEFT",
    "BTN_RIGHT",
    "BTN_MIDDLE",
    "BTN_SIDE",
    "BTN_EXTRA",
    "BTN_FORWARD",
    "BTN_BACK",
    "BTN_TASK",
];

// Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/output.py#L10-L32
pub fn output_device() -> Result<VirtualDevice, Box<dyn Error>> {
    let mut keys: AttributeSet<Key> = AttributeSet::new();
    for code in Key::KEY_RESERVED.code()..Key::BTN_TRIGGER_HAPPY40.code() {
        let key = Key::new(code);
        let name = format!("{:?}", key);
        if name.starts_with("KEY_") || MOUSE_BTNS.contains(&&**&name) {
            keys.insert(key);
        }
    }

    let mut relative_axes: AttributeSet<RelativeAxisType> = AttributeSet::new();
    relative_axes.insert(RelativeAxisType::REL_X);
    relative_axes.insert(RelativeAxisType::REL_Y);
    relative_axes.insert(RelativeAxisType::REL_HWHEEL);
    relative_axes.insert(RelativeAxisType::REL_WHEEL);
    relative_axes.insert(RelativeAxisType::REL_MISC);

    let device = VirtualDeviceBuilder::new()?
        .name(&current_device_name())
        .with_keys(&keys)?
        .with_relative_axes(&relative_axes)?
        .build()?;
    Ok(device)
}

pub fn device_watcher(watch: bool) -> Result<Option<Inotify>, Box<dyn Error>> {
    if watch {
        let inotify = Inotify::init(InitFlags::empty())?;
        inotify.add_watch("/dev/input", AddWatchFlags::IN_CREATE | AddWatchFlags::IN_ATTRIB)?;
        Ok(Some(inotify))
    } else {
        Ok(None)
    }
}

pub fn input_devices(
    device_opts: &Vec<String>,
    ignore_opts: &Vec<String>,
    watch: bool,
) -> Result<Vec<Device>, Box<dyn Error>> {
    let mut path_devices = list_devices()?;
    let mut paths: Vec<String> = path_devices.keys().map(|e| e.clone()).collect();
    paths.sort_by(|a, b| device_index(a).partial_cmp(&device_index(b)).unwrap());

    println!("Selecting devices from the following list:");
    println!("{}", SEPARATOR);
    for path in &paths {
        if let Some(device) = path_devices.get(path) {
            println!("{:18}: {}", path, device_name(device));
        }
    }
    println!("{}", SEPARATOR);

    if device_opts.is_empty() {
        print!("Selected keyboards automatically since --device options weren't specified");
    } else {
        print!("Selected devices matching {:?}", device_opts);
    };
    if ignore_opts.is_empty() {
        println!(":")
    } else {
        println!(", ignoring {:?}:", ignore_opts);
    }
    for path in &paths {
        if let Some(device) = path_devices.get(path) {
            let matched = if device_opts.is_empty() {
                is_keyboard(device)
            } else {
                match_device(path, device, device_opts)
            } && (ignore_opts.is_empty() || !match_device(path, device, ignore_opts));
            if !matched {
                path_devices.remove(path);
            }
        }
    }

    println!("{}", SEPARATOR);
    if path_devices.is_empty() {
        if watch {
            println!("warning: No device was selected, but --watch is waiting for new devices.");
        } else {
            return Err("No device was selected!".into());
        }
    } else {
        for (path, device) in path_devices.iter() {
            println!("{:18}: {}", path, device_name(device));
        }
    }
    println!("{}", SEPARATOR);

    let mut devices: Vec<Device> = path_devices.into_values().collect();
    for device in devices.iter_mut() {
        device
            .grab()
            .map_err(|e| format!("Failed to grab device '{}': {}", device_name(device), e))?;
    }
    return Ok(devices);
}

// We can't know the device path from evdev::enumerate(). So we re-implement it.
fn list_devices() -> Result<HashMap<String, Device>, Box<dyn Error>> {
    let mut path_devices: HashMap<String, Device> = HashMap::new();
    if let Some(dev_input) = read_dir("/dev/input").as_mut().ok() {
        while let Some(entry) = dev_input.next() {
            let path = entry?.path();
            if let Some(fname) = path.file_name() {
                if fname.as_bytes().starts_with(b"event") {
                    // Allow "Permission denied" when opening the current process's own device.
                    if let Ok(device) = Device::open(&path) {
                        if let Ok(path) = path.into_os_string().into_string() {
                            path_devices.insert(path, device);
                        }
                    }
                }
            }
        }
    }
    return Ok(path_devices);
}

fn device_name(device: &Device) -> &str {
    device.name().unwrap_or("<Unnamed device>")
}

fn device_index(path: &str) -> i32 {
    path.trim_start_matches("/dev/input/event").parse::<i32>().unwrap()
}

fn current_device_name() -> String {
    format!("xremap pid={}", process::id())
}

fn match_device(path: &str, device: &Device, device_opts: &Vec<String>) -> bool {
    // Force unmatch its own device
    if device_name(device) == &current_device_name() {
        return false;
    }

    for device_opt in device_opts {
        // Check exact matches for explicit selection
        if path == device_opt || device_name(device) == device_opt {
            return true;
        }
        // eventXX shorthand for /dev/input/eventXX
        if device_opt.starts_with("event") && path == format!("/dev/input/{}", device_opt) {
            return true;
        }
        // Allow partial matches for device names
        if device_name(device).contains(device_opt) {
            return true;
        }
    }
    return false;
}

fn is_keyboard(device: &Device) -> bool {
    // Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/input.py#L17-L32
    match device.supported_keys() {
        Some(keys) => {
            keys.contains(Key::KEY_SPACE)
                && keys.contains(Key::KEY_A)
                && keys.contains(Key::KEY_Z)
                // BTN_MOUSE
                && !keys.contains(Key::BTN_LEFT)
        }
        None => false,
    }
}

static SEPARATOR: &str = "------------------------------------------------------------------------------";
