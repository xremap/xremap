use evdev::Key;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::str::FromStr;

pub fn deserialize_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let key = String::deserialize(deserializer)?;
    parse_key(&key).map_err(serde::de::Error::custom)
}

pub fn parse_key(input: &str) -> Result<Key, Box<dyn Error>> {
    // Everything is case-insensitive
    let name = input.to_uppercase();

    // Original evdev scancodes should always work
    if let Ok(key) = Key::from_str(&name) {
        return Ok(key);
    }

    // You can abbreviate "KEY_" of any "KEY_*" scancodes.
    if let Ok(key) = Key::from_str(&format!("KEY_{}", name)) {
        return Ok(key);
    }

    // xremap's custom aliases like k0kubun/karabiner-dsl
    let key = match &name[..] {
        // Shift
        "SHIFT_R" => Key::KEY_RIGHTSHIFT,
        "SHIFT_L" => Key::KEY_LEFTSHIFT,
        // Control
        "CONTROL_R" => Key::KEY_RIGHTCTRL,
        "CONTROL_L" => Key::KEY_LEFTCTRL,
        "CTRL_R" => Key::KEY_RIGHTCTRL,
        "CTRL_L" => Key::KEY_LEFTCTRL,
        "C_R" => Key::KEY_RIGHTCTRL,
        "C_L" => Key::KEY_LEFTCTRL,
        // Alt
        "ALT_R" => Key::KEY_RIGHTALT,
        "ALT_L" => Key::KEY_LEFTALT,
        "M_R" => Key::KEY_RIGHTALT,
        "M_L" => Key::KEY_LEFTALT,
        // Windows
        "SUPER_R" => Key::KEY_RIGHTMETA,
        "SUPER_L" => Key::KEY_LEFTMETA,
        "WIN_R" => Key::KEY_RIGHTMETA,
        "WIN_L" => Key::KEY_LEFTMETA,

        //Entirely custom scancodes
        //
        //Cursor movement
        "BTN_XRIGHTCURSOR" => evdev::Key(59974), //Cursor right
        "BTN_XLEFTCURSOR" => evdev::Key(59975),  //Cursor left
        "BTN_XDOWNCURSOR" => evdev::Key(59976),  //Cursor down
        "BTN_XUPCURSOR" => evdev::Key(59977),    //Cursor up
        //Cursor... forward and backwards?
        "BTN_XREL_Z_AXIS_1" => evdev::Key(59978),
        "BTN_XREL_Z_AXIS_2" => evdev::Key(59979),
        //
        //Rotative cursor movement?
        "BTN_XREL_RX_AXIS_1" => evdev::Key(59980), //horizontal
        "BTN_XREL_RX_AXIS_2" => evdev::Key(59981),
        "BTN_XREL_RY_AXIS_1" => evdev::Key(59982), //vertical
        "BTN_XREL_RY_AXIS_2" => evdev::Key(59983),
        "BTN_XREL_RZ_AXIS_1" => evdev::Key(59984), //Whatever the third dimensional axis is called
        "BTN_XREL_RZ_AXIS_2" => evdev::Key(59985),
        //
        "BTN_XRIGHTSCROLL" => evdev::Key(59986), //Rightscroll
        "BTN_XLEFTSCROLL" => evdev::Key(59987),  //Leftscroll
        //
        //???
        "BTN_XREL_DIAL_1" => evdev::Key(59988),
        "BTN_XREL_DIAL_2" => evdev::Key(59989),
        //
        "BTN_XUPSCROLL" => evdev::Key(59990),   //Upscroll
        "BTN_XDOWNSCROLL" => evdev::Key(59991), //Downscroll
        //
        //Something?
        "BTN_XREL_MISC_1" => evdev::Key(59992),
        "BTN_XREL_MISC_2" => evdev::Key(59993),
        "BTN_XREL_RESERVED_1" => evdev::Key(59994),
        "BTN_XREL_RESERVED_2" => evdev::Key(59995),
        //
        //High resolution version of scroll events, sent just after their non-high resolution version.
        "BTN_XHIRES_UPSCROLL" => evdev::Key(59996),
        "BTN_XHIRES_DOWNSCROLL" => evdev::Key(59997),
        "BTN_XHIRES_RIGHTSCROLL" => evdev::Key(59998),
        "BTN_XHIRES_LEFTSCROLL" => evdev::Key(59999),
        /* Original Relative events and their values for quick reference.
            REL_X = 0x00,
            REL_Y = 0x01,
            REL_Z = 0x02,
            REL_RX = 0x03,
            REL_RY = 0x04,
            REL_RZ = 0x05,
            REL_HWHEEL = 0x06,
            REL_DIAL = 0x07,
            REL_WHEEL = 0x08,
            REL_MISC = 0x09,
            REL_RESERVED = 0x0a,
            REL_WHEEL_HI_RES = 0x0b,
            REL_HWHEEL_HI_RES = 0x0c,
        */
        //End of custom scancodes

        // else
        _ => Key::KEY_RESERVED,
    };
    if key != Key::KEY_RESERVED {
        return Ok(key);
    }

    return Err(format!("unknown key '{}'", input).into());
}
