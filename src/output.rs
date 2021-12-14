use std::error::Error;
use evdev::{AttributeSet, Device, RelativeAxisType};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};

pub fn build_device(base_device: &Device) -> Result<VirtualDevice, Box<dyn Error>> {
    let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
    rel_axes.insert(evdev::RelativeAxisType::REL_X);
    rel_axes.insert(evdev::RelativeAxisType::REL_Y);
    rel_axes.insert(evdev::RelativeAxisType::REL_HWHEEL);
    rel_axes.insert(evdev::RelativeAxisType::REL_WHEEL);
    rel_axes.insert(evdev::RelativeAxisType::REL_MISC);

    let device = VirtualDeviceBuilder::new()?
        .name("xremap")
        .with_keys(base_device.supported_keys().unwrap())?
        .with_relative_axes(&rel_axes)?
        .build()
        .unwrap();
    Ok(device)
}
