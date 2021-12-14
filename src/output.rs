use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{Device};
use std::error::Error;

pub fn build_device(base_device: &Device) -> Result<VirtualDevice, Box<dyn Error>> {
    let device = VirtualDeviceBuilder::new()?
        .name("xremap")
        .with_keys(base_device.supported_keys().unwrap())?
        .build()
        .unwrap();

    Ok(device)
}
