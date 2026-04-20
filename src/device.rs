use anyhow::bail;
use derive_where::derive_where;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, BusType, Device, FetchEventsSynced, InputId, KeyCode as Key, RelativeAxisCode};
use log::debug;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::collections::HashMap;
#[cfg(feature = "udev")]
use std::fs::metadata;
use std::fs::{self, read_dir};
#[cfg(feature = "udev")]
use std::os::linux::fs::MetadataExt;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::prelude::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{io, process};
#[cfg(feature = "udev")]
use udev::DeviceType;

use crate::util::{evdev_enums_to_string, print_table};

pub fn choose_device_name() -> String {
    let name_already_taken = match input_devices() {
        Ok(devices) => devices.iter().any(|device| device.device_name().contains("xremap")),
        Err(_) => true, // fallback to the safe side
    };

    if name_already_taken {
        format!("xremap pid={}", process::id())
    } else {
        "xremap".to_string()
    }
}

// Credit: https://github.com/mooz/xkeysnail/blob/bf3c93b4fe6efd42893db4e6588e5ef1c4909cfb/xkeysnail/output.py#L10-L32
pub fn output_device(
    bus_type: Option<BusType>,
    enable_wheel: bool,
    vendor: u16,
    product: u16,
    own_device: &str,
) -> anyhow::Result<VirtualDevice> {
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
        .name(own_device)
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

// We can't know the device path from evdev::enumerate(). So we re-implement it.
fn input_devices() -> anyhow::Result<Vec<InputDevice>> {
    Ok(read_dir("/dev/input")
        .map_err(|err| anyhow::format_err!("Failed to read /dev/input: {err}"))?
        .filter_map(|entry| {
            // Allow "Permission denied" when opening the current process's own device.
            open_device(entry.ok()?.path())
        })
        .collect())
}

pub fn select_input_devices(
    device_opts: &[String],
    ignore_opts: &[String],
    mouse: bool,
    watch: bool,
    own_device: &str,
) -> anyhow::Result<HashMap<PathBuf, InputDevice>> {
    let mut devices = input_devices()?;
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
    println!("{SEPARATOR}");

    let mut selected: Vec<InputDevice> = vec![];
    for mut device in devices.into_iter() {
        if device.is_input_device(device_opts, ignore_opts, mouse, own_device) && device.grab() {
            device.print();
            selected.push(device)
        }
    }

    if selected.is_empty() {
        if watch {
            println!("No device was selected, but --watch is waiting for new devices.");
        } else {
            bail!("Failed to prepare input devices: No device was selected!");
        }
    }
    println!("{SEPARATOR}");

    Ok(selected.into_iter().map(From::from).collect())
}

pub fn open_device(path: PathBuf) -> Option<InputDevice> {
    path.file_name()?
        .as_bytes()
        .starts_with(b"event")
        .then_some(InputDevice {
            device: Device::open(&path).ok()?,
            path,
        })
}

#[derive(Debug, Clone)]
pub struct InputDeviceInfo {
    pub name: String,
    pub path: PathBuf,
    pub product: u16,
    pub vendor: u16,
}

impl InputDeviceInfo {
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
        // Match udev symlinks to actual physical device path
        if Path::new(filter).is_absolute() {
            if let Ok(resolved_filter) = fs::canonicalize(filter) {
                if self.path == resolved_filter {
                    return true;
                }
            }
        }

        #[cfg(feature = "udev")]
        {
            if filter.starts_with("props:") {
                if let Ok(meta) = metadata(&self.path) {
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
        #[cfg(not(feature = "device-test"))]
        let count = 50;
        #[cfg(feature = "device-test")]
        let count = 2;

        for _ in 0..count {
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

    pub fn device_name(&self) -> &str {
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

    pub fn to_info(&self) -> InputDeviceInfo {
        InputDeviceInfo {
            name: self.device_name().into(),
            product: self.product(),
            vendor: self.vendor(),
            path: self.path.clone(),
        }
    }

    pub fn is_input_device(
        &self,
        device_filter: &[String],
        ignore_filter: &[String],
        mouse: bool,
        own_device: &str,
    ) -> bool {
        if self.device_name() == own_device {
            return false;
        }
        (if device_filter.is_empty() {
            self.is_keyboard() || (mouse && self.is_mouse())
        } else {
            self.matches_any(device_filter)
        }) && (ignore_filter.is_empty() || !self.matches_any(ignore_filter))
    }

    fn matches_any(&self, filter: &[String]) -> bool {
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

    pub fn print_details(&self) {
        let properties = evdev_enums_to_string(self.device.properties());
        let events = evdev_enums_to_string(self.device.supported_events());
        let keys = evdev_enums_to_string(self.device.supported_keys().unwrap_or_default());
        let relative = evdev_enums_to_string(self.device.supported_relative_axes().unwrap_or_default());
        let absolute = evdev_enums_to_string(self.device.supported_absolute_axes().unwrap_or_default());
        let leds = evdev_enums_to_string(self.device.supported_leds().unwrap_or_default());
        let switches = evdev_enums_to_string(self.device.supported_switches().unwrap_or_default());

        println!("{}", self.device_name());
        println!("");
        println!("  Path:            {}", self.path.display());
        println!("  Type:            {}", self.bus_type());
        println!(
            "  Vendor/product:  {}:{} (0x{:x}:0x{:x}) ",
            self.vendor(),
            self.product(),
            self.vendor(),
            self.product()
        );
        println!("  Properties:      {}", properties);
        println!("  Events:          {}", events);
        println!("  Keys:            {}", keys);
        println!("  Relative axes:   {}", relative);
        println!("  Absolute axes:   {}", absolute);
        println!("  Leds:            {}", leds);
        println!("  Switches:        {}", switches);
        println!("");
    }
}

/// List info about devices
pub fn print_device_list() -> anyhow::Result<()> {
    let mut devices = input_devices()?;
    devices.sort();

    let mut table: Vec<Vec<String>> = vec![];

    table.push(vec![
        "PATH".into(),
        "NAME".into(),
        "IS_KEYBOARD".into(),
        "IS_MOUSE".into(),
        "TYPE".into(),
        "VENDOR".into(),
        "PRODUCT".into(),
    ]);

    for device in devices {
        table.push(vec![
            device.path.display().to_string(),
            device.device_name().to_string(),
            format!("{}", device.is_keyboard()),
            format!("{}", device.is_mouse()),
            device.bus_type().to_string(),
            format!("0x{:x}", device.vendor()),
            format!("0x{:x}", device.product()),
        ]);
    }

    print_table(table);

    Ok(())
}

/// Show device details
pub fn print_device_details() -> anyhow::Result<()> {
    let mut devices = input_devices()?;
    devices.sort();

    for device in devices {
        device.print_details();
    }

    Ok(())
}

pub const SEPARATOR: &str = "------------------------------------------------------------------------------";
