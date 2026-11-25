mod config_watcher;
mod device_watcher;

pub use config_watcher::ConfigWatcher;
pub use device_watcher::DeviceWatcher;

pub const ENODEV_EVDEV: i32 = 6; // When fetching from a closed device
pub const ENODEV_UNGRAB: i32 = 25; // When ungrabbing closed device
