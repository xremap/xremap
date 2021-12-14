use std::error::Error;
use evdev::{Device};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};

pub fn build_device(base_device: &Device) -> Result<VirtualDevice, Box<dyn Error>> {
    let device = VirtualDeviceBuilder::new()?
        .name("xremap")
        .with_keys(base_device.supported_keys().unwrap())?
        .build()
        .unwrap();

    Ok(device)
}
