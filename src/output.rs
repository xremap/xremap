use std::error::Error;
use evdev::{AttributeSet, Key};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};

pub fn build_device() -> Result<VirtualDevice, Box<dyn Error>> {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::KEY_A);

    let device = VirtualDeviceBuilder::new()?
        .name("xremap")
        .with_keys(&keys)?
        .build()
        .unwrap();

    Ok(device)
}
