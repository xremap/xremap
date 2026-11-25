mod config_watcher;
mod device_watcher;

pub use config_watcher::ConfigWatcher;
pub use device_watcher::DeviceWatcher;

// FreeBSD has different values for these
pub const ENODEV_EVDEV: i32 = nix::libc::ENODEV;
pub const ENODEV_UNGRAB: i32 = nix::libc::ENODEV;
