use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Key, RelativeAxisType};
use std::error::Error;

static MOUSE_BTNS: [&str; 13] = [
    "BTN_0",
    "BTN_1",
    "BTN_2",
    "BTN_3",
    "BTN_4",
    "BTN_5",
    "BTN_6",
    "BTN_7",
    "BTN_8",
    "BTN_9",
    "BTN_LEFT",
    "BTN_MIDDLE",
    "BTN_RIGHT",
];

// Credit: https://github.com/mooz/xkeysnail/blob/master/xkeysnail/output.py#L10-L32
pub fn build_device() -> Result<VirtualDevice, Box<dyn Error>> {
    let mut keys: AttributeSet<Key> = AttributeSet::new();
    for code in Key::KEY_RESERVED.code()..Key::BTN_TRIGGER_HAPPY40.code() {
        let key = Key::new(code);
        let name = format!("{:?}", key);
        if name.starts_with("KEY_") || MOUSE_BTNS.contains(&&**&name) {
            keys.insert(key);
        }
    }

    let mut relative_axes: AttributeSet<RelativeAxisType> = AttributeSet::new();
    relative_axes.insert(RelativeAxisType::REL_X);
    relative_axes.insert(RelativeAxisType::REL_Y);
    relative_axes.insert(RelativeAxisType::REL_HWHEEL);
    relative_axes.insert(RelativeAxisType::REL_WHEEL);
    relative_axes.insert(RelativeAxisType::REL_MISC);

    let device = VirtualDeviceBuilder::new()?
        .name("xremap")
        .with_keys(&keys)?
        .with_relative_axes(&relative_axes)?
        .build()?;
    Ok(device)
}
