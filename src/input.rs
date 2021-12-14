extern crate evdev;

use evdev::Device;

pub fn select_device() -> Device {
    // TODO: stop hard-coding the device
    let device = Device::open("/dev/input/event2");
    return device.unwrap();
}
