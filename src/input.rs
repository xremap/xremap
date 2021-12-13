extern crate evdev;

use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem, ptr};
use evdev::Device;

pub fn select_device() -> Device {
    // TODO: stop hard-coding the device
    let device = Device::open("/dev/input/event2");
    return device.unwrap()
}
