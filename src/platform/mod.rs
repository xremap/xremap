#[cfg(target_os = "freebsd")]
mod platform_freebsd;
#[cfg(target_os = "linux")]
mod platform_linux;

#[cfg(target_os = "freebsd")]
pub use platform_freebsd::{ConfigWatcher, DeviceWatcher};
#[cfg(target_os = "linux")]
pub use platform_linux::{ConfigWatcher, DeviceWatcher};
