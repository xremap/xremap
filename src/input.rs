extern crate evdev;

use evdev::Device;
use std::error::Error;

pub fn select_device() -> Result<Device, Box<dyn Error>> {
    // TODO: stop hard-coding the device
    let device = Device::open("/dev/input/event19")?;
    return Ok(device);
}
