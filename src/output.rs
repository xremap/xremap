use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{Device};
use std::error::Error;

pub fn build_device(base_device: &Device) -> Result<VirtualDevice, Box<dyn Error>> {
    let builder = VirtualDeviceBuilder::new()?.name("xremap");
    let device = match base_device.supported_keys() {
        Some(keys) => builder.with_keys(keys)?,
        None => builder,
    }.build()?;
    Ok(device)
}
