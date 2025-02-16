extern crate evdev;
extern crate nix;

use anyhow::bail;
use derive_where::derive_where;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, BusType, Device, FetchEventsSynced, InputId, Key, RelativeAxisType};
use log::debug;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::prelude::AsRawFd;
use std::path::{Path, PathBuf};
use std::{io, process};

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

static mut DEVICE_NAME: Option<String> = None;

// Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/output.py#L10-L32
pub fn output_device(bus_type: Option<BusType>, enable_wheel: bool) -> Result<VirtualDevice, Box<dyn Error>> {
    let mut keys: AttributeSet<Key> = AttributeSet::new();
    for code in Key::KEY_RESERVED.code()..Key::BTN_TRIGGER_HAPPY40.code() {
        let key = Key::new(code);
        let name = format!("{:?}", key);
        if name.starts_with("KEY_") || MOUSE_BTNS.contains(&&*name) {
            keys.insert(key);
        }
    }

    let mut relative_axes: AttributeSet<RelativeAxisType> = AttributeSet::new();
    relative_axes.insert(RelativeAxisType::REL_X);
    relative_axes.insert(RelativeAxisType::REL_Y);
    if enable_wheel {
        relative_axes.insert(RelativeAxisType::REL_HWHEEL);
        relative_axes.insert(RelativeAxisType::REL_WHEEL);
    }
    relative_axes.insert(RelativeAxisType::REL_MISC);

    let device = VirtualDeviceBuilder::new()?
        // These are taken from https://docs.rs/evdev/0.12.0/src/evdev/uinput.rs.html#183-188
        .input_id(InputId::new(bus_type.unwrap_or(BusType::BUS_USB), 0x1234, 0x5678, 0x111))
        .name(&InputDevice::current_name())
        .with_keys(&keys)?
        .with_relative_axes(&relative_axes)?
        .build()?;
    Ok(device)
}

pub fn device_watcher(watch: bool) -> anyhow::Result<Option<Inotify>> {
    if watch {
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
        inotify.add_watch("/dev/input", AddWatchFlags::IN_CREATE | AddWatchFlags::IN_ATTRIB)?;
        Ok(Some(inotify))
    } else {
        Ok(None)
    }
}

pub fn get_input_devices(
    device_opts: &[String],
    ignore_opts: &[String],
    mouse: bool,
    watch: bool,
) -> anyhow::Result<HashMap<PathBuf, InputDevice>> {
    let mut devices: Vec<_> = InputDevice::devices()?.collect();
    devices.sort();

    println!("Selecting devices from the following list:");
    println!("{}", SEPARATOR);
    devices.iter().for_each(InputDevice::print);
    println!("{}", SEPARATOR);

    if device_opts.is_empty() {
        if mouse {
            print!("Selected keyboards and mice automatically since --device options weren't specified");
        } else {
            print!("Selected keyboards automatically since --device options weren't specified");
        }
    } else {
        print!("Selected devices matching {:?}", device_opts);
    };
    if ignore_opts.is_empty() {
        println!(":")
    } else {
        println!(", ignoring {:?}:", ignore_opts);
    }

    let devices: Vec<_> = devices
        .into_iter()
        // filter map needed for mutable access
        // alternative is `Vec::retain_mut` whenever that gets stabilized
        .filter_map(|mut device| {
            // filter out any not matching devices and devices that error on grab
            (device.is_input_device(device_opts, ignore_opts, mouse) && device.grab()).then(|| device)
        })
        .collect();

    println!("{}", SEPARATOR);
    if devices.is_empty() {
        if watch {
            println!("warning: No device was selected, but --watch is waiting for new devices.");
        } else {
            bail!("No device was selected!");
        }
    } else {
        devices.iter().for_each(InputDevice::print);
    }
    println!("{}", SEPARATOR);

    Ok(devices.into_iter().map(From::from).collect())
}

#[derive(Debug)]
pub struct InputDeviceInfo<'a> {
    pub name: &'a str,
    pub path: &'a Path,
}

impl<'a> InputDeviceInfo<'a> {
    pub fn matches(&self, filter: &String) -> bool {
        let filter = filter.as_str();
        // Check exact matches for explicit selection
        if self.path.as_os_str() == filter || self.name == filter {
            return true;
        }
        // eventXX shorthand for /dev/input/eventXX
        if filter.starts_with("event") && self.path.file_name().expect("every device path has a file name") == filter {
            return true;
        }
        // Allow partial matches for device names
        if self.name.contains(filter) {
            return true;
        }
        return false;
    }
}

#[derive_where(PartialEq, PartialOrd, Ord)]
pub struct InputDevice {
    path: PathBuf,
    #[derive_where(skip)]
    device: Device,
}

impl Eq for InputDevice {}

impl TryFrom<PathBuf> for InputDevice {
    type Error = io::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let fname = path
            .file_name()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        if fname.as_bytes().starts_with(b"event") {
            Ok(Self {
                device: Device::open(&path)?,
                path,
            })
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }
}

impl From<InputDevice> for (PathBuf, InputDevice) {
    fn from(device: InputDevice) -> Self {
        (device.path.clone(), device)
    }
}

impl AsRawFd for InputDevice {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.device.as_raw_fd()
    }
}

/// Device Wrappers Abstractions
impl InputDevice {
    pub fn grab(&mut self) -> bool {
        if let Err(error) = self.device.grab() {
            println!("Failed to grab device '{}' at '{}' due to: {error}", self.device_name(), self.path.display());
            false
        } else {
            true
        }
    }

    pub fn ungrab(&mut self) {
        if let Err(error) = self.device.ungrab() {
            println!("Failed to ungrab device '{}' at '{}' due to: {error}", self.device_name(), self.path.display());
        }
    }

    pub fn fetch_events(&mut self) -> io::Result<FetchEventsSynced> {
        self.device.fetch_events()
    }

    fn device_name(&self) -> &str {
        self.device.name().unwrap_or("<Unnamed device>")
    }

    pub fn bus_type(&self) -> BusType {
        self.device.input_id().bus_type()
    }

    pub fn to_info(&self) -> InputDeviceInfo {
        InputDeviceInfo {
            name: self.device_name(),
            path: &self.path,
        }
    }
}

impl InputDevice {
    pub fn is_input_device(&self, device_filter: &[String], ignore_filter: &[String], mouse: bool) -> bool {
        if self.device_name() == Self::current_name() {
            return false;
        }
        (if device_filter.is_empty() {
            self.is_keyboard() || (mouse && self.is_mouse())
        } else {
            self.matches_any(device_filter)
        }) && (ignore_filter.is_empty() || !self.matches_any(ignore_filter))
    }

    // We can't know the device path from evdev::enumerate(). So we re-implement it.
    fn devices() -> io::Result<impl Iterator<Item = InputDevice>> {
        Ok(read_dir("/dev/input")?.filter_map(|entry| {
            // Allow "Permission denied" when opening the current process's own device.
            InputDevice::try_from(entry.ok()?.path()).ok()
        }))
    }

    #[allow(static_mut_refs)]
    fn current_name() -> &'static str {
        if unsafe { DEVICE_NAME.is_none() } {
            let device_name = if Self::has_device_name("xremap") {
                format!("xremap pid={}", process::id())
            } else {
                "xremap".to_string()
            };
            unsafe {
                DEVICE_NAME = Some(device_name);
            }
        }
        unsafe { DEVICE_NAME.as_ref() }.unwrap()
    }

    fn has_device_name(device_name: &str) -> bool {
        let devices: Vec<_> = match Self::devices() {
            Ok(devices) => devices.collect(),
            Err(_) => return true, // fallback to the safe side
        };
        devices
            .iter()
            .any(|device| return device.device_name().contains(device_name))
    }

    fn matches_any(&self, filter: &[String]) -> bool {
        // Force unmatch its own device
        if self.device_name() == Self::current_name() {
            return false;
        }
        return filter.iter().any(|f| self.to_info().matches(f));
    }

    fn is_keyboard(&self) -> bool {
        // Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/input.py#L17-L32
        match self.device.supported_keys() {
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

    fn is_mouse(&self) -> bool {
        // Xremap doesn't support absolute device so will break them.
        if self.device.supported_absolute_axes().is_some()  {
            debug!("Ignoring absolute device {:18} {}", self.path.display(), self.device_name());
            return false;
        }
        self.device
            .supported_keys()
            .map_or(false, |keys| keys.contains(Key::BTN_LEFT))
    }

    pub fn print(&self) {
        println!("{:18}: {}", self.path.display(), self.device_name())
    }
}

const SEPARATOR: &str = "------------------------------------------------------------------------------";
