extern crate evdev;
extern crate nix;

use evdev::Device;
use std::error::Error;
use nix::sys::select::FdSet;
use nix::sys::select::select;
use std::os::unix::io::{AsRawFd};

pub fn select_device() -> Result<Device, Box<dyn Error>> {
    // TODO: stop hard-coding the device
    let device = Device::open("/dev/input/event19")?;
    return Ok(device);
}

pub fn is_readable(device: &mut Device) -> Result<bool, Box<dyn Error>> {
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
