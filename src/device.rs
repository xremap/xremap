extern crate evdev;
extern crate nix;

use anyhow::bail;
use derive_where::derive_where;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, BusType, Device, FetchEventsSynced, InputId, KeyCode as Key, RelativeAxisCode};
use log::debug;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::collections::HashMap;
use std::error::Error;
#[cfg(feature = "udev")]
use std::fs::metadata;
use std::fs::read_dir;
#[cfg(feature = "udev")]
use std::os::linux::fs::MetadataExt;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::prelude::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{io, process};
#[cfg(feature = "udev")]
use udev::DeviceType;

pub static mut DEVICE_NAME: Option<String> = None;

// Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/output.py#L10-L32
pub fn output_device(
    bus_type: Option<BusType>,
    enable_wheel: bool,
    vendor: u16,
    product: u16,
) -> Result<VirtualDevice, Box<dyn Error>> {
    let mut keys: AttributeSet<Key> = AttributeSet::new();
    for code in Key::KEY_RESERVED.code()..Key::BTN_TRIGGER_HAPPY40.code() {
        let key = Key::new(code);
        let name = format!("{key:?}");
        if name.starts_with("KEY_") || name.starts_with("BTN_") {
            keys.insert(key);
        }
    }

    let mut relative_axes: AttributeSet<RelativeAxisCode> = AttributeSet::new();
    relative_axes.insert(RelativeAxisCode::REL_X);
    relative_axes.insert(RelativeAxisCode::REL_Y);
    if enable_wheel {
        relative_axes.insert(RelativeAxisCode::REL_HWHEEL);
        relative_axes.insert(RelativeAxisCode::REL_WHEEL);
    }
    relative_axes.insert(RelativeAxisCode::REL_MISC);

    let device = VirtualDevice::builder()?
        // These are taken from https://docs.rs/evdev/0.12.0/src/evdev/uinput.rs.html#183-188
        .input_id(InputId::new(bus_type.unwrap_or(BusType::BUS_USB), vendor, product, 0x111))
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
    println!("{SEPARATOR}");
    devices.iter().for_each(InputDevice::print);
    println!("{SEPARATOR}");

    if device_opts.is_empty() {
        if mouse {
            print!("Selected keyboards and mice automatically since --device options weren't specified");
        } else {
            print!("Selected keyboards automatically since --device options weren't specified");
        }
    } else {
        print!("Selected devices matching {device_opts:?}");
    };
    if ignore_opts.is_empty() {
        println!(":")
    } else {
        println!(", ignoring {ignore_opts:?}:");
    }

    let devices: Vec<_> = devices
        .into_iter()
        // filter map needed for mutable access
        // alternative is `Vec::retain_mut` whenever that gets stabilized
        .filter_map(|mut device| {
            // filter out any not matching devices and devices that error on grab
            (device.is_input_device(device_opts, ignore_opts, mouse) && device.grab()).then_some(device)
        })
        .collect();

    println!("{SEPARATOR}");
    if devices.is_empty() {
        if watch {
            println!("warning: No device was selected, but --watch is waiting for new devices.");
        } else {
            bail!("No device was selected!");
        }
    } else {
        devices.iter().for_each(InputDevice::print);
    }
    println!("{SEPARATOR}");

    Ok(devices.into_iter().map(From::from).collect())
}

#[derive(Debug)]
pub struct InputDeviceInfo<'a> {
    pub name: &'a str,
    pub path: &'a Path,
    pub product: u16,
    pub vendor: u16,
}

impl<'a> InputDeviceInfo<'a> {
    pub fn matches(&self, filter: &str) -> bool {
        // Check exact matches for explicit selection
        if self.path.as_os_str() == filter || self.name == filter {
            return true;
        }
        // eventXX shorthand for /dev/input/eventXX
        if filter.starts_with("event") && self.path.file_name().expect("every device path has a file name") == filter {
            return true;
        }
        if filter.starts_with("ids:") {
            let args = filter.split(':').collect::<Vec<&str>>();
            if args.len() == 3 {
                let vid = u16::from_str_radix(args[1].trim_start_matches("0x"), 16).unwrap_or(0);
                let pid = u16::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap_or(0);
                match (vid, pid) {
                    (0, 0) => {}
                    (v, 0) if v == self.vendor => {
                        return true;
                    }
                    (0, p) if p == self.product => {
                        return true;
                    }
                    (v, p) if v == self.vendor && p == self.product => {
                        return true;
                    }
                    (_, _) => {}
                }
            }
        }
        // Allow partial matches for device names
        if self.name.contains(filter) {
            return true;
        }

        #[cfg(feature = "udev")]
        {
            if filter.starts_with("props:") {
                if let Ok(meta) = metadata(self.path) {
                    let args = filter.split(':').collect::<Vec<&str>>();
                    if args.len() == 3 {
                        if let Ok(ud) = udev::Device::from_devnum(DeviceType::Character, meta.st_rdev()) {
                            for _ in 0..10 {
                                if ud.is_initialized() {
                                    break;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                            let props = ud.properties();
                            for p in props.filter(|p| p.name() == args[1]) {
                                if p.value() == args[2] {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
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
    pub fn wait_for_all_keys_up(&self) -> io::Result<()> {
        for _ in 0..50 {
            let keys = self.device.get_key_state()?;

            if keys.iter().filter(|&key| key != Key::KEY_UNKNOWN).count() == 0 {
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        Err(io::Error::new(io::ErrorKind::TimedOut, "Timed out waiting for keys to be released."))
    }

    pub fn grab(&mut self) -> bool {
        let result = self.wait_for_all_keys_up().and_then(|_| self.device.grab());

        match result {
            Ok(_) => true,
            Err(error) => {
                eprintln!(
                    "warning: Failed to grab device '{}' at '{}'. It may have been disconnected, have keys held down, or you may need to grant permissions. Error: {}",
                    self.device_name(),
                    self.path.display(),
                    error
                );
                false
            }
        }
    }

    pub fn ungrab(&mut self) {
        if let Err(error) = self.device.ungrab() {
            println!("Failed to ungrab device '{}' at '{}' due to: {error}", self.device_name(), self.path.display());
        }
    }

    pub fn fetch_events(&mut self) -> io::Result<FetchEventsSynced<'_>> {
        self.device.fetch_events()
    }

    fn device_name(&self) -> &str {
        self.device.name().unwrap_or("<Unnamed device>")
    }

    pub fn bus_type(&self) -> BusType {
        self.device.input_id().bus_type()
    }

    pub fn product(&self) -> u16 {
        self.device.input_id().product()
    }

    pub fn vendor(&self) -> u16 {
        self.device.input_id().vendor()
    }

    pub fn to_info(&self) -> InputDeviceInfo<'_> {
        InputDeviceInfo {
            name: self.device_name(),
            product: self.product(),
            vendor: self.vendor(),
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
        devices.iter().any(|device| device.device_name().contains(device_name))
    }

    fn matches_any(&self, filter: &[String]) -> bool {
        // Force unmatch its own device
        if self.device_name() == Self::current_name() {
            return false;
        }
        filter.iter().any(|f| self.to_info().matches(f))
    }

    fn is_keyboard(&self) -> bool {
        // Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/input.py#L17-L32
        match self.device.supported_keys() {
            Some(keys) => keys.contains(Key::KEY_SPACE) && keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z),
            None => false,
        }
    }

    fn is_mouse(&self) -> bool {
        // Xremap doesn't support absolute device so will break them.
        if self.device.supported_absolute_axes().is_some() {
            debug!("Ignoring absolute device {:18} {}", self.path.display(), self.device_name());
            return false;
        }
        self.device
            .supported_keys()
            .is_some_and(|keys| keys.contains(Key::BTN_LEFT))
    }

    pub fn print(&self) {
        println!("{:18}: {}", self.path.display(), self.device_name())
    }
}

pub const SEPARATOR: &str = "------------------------------------------------------------------------------";
