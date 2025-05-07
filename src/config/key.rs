use crate::event_handler::{DISGUISED_EVENT_OFFSETTER, KEY_MATCH_ANY};
use evdev::KeyCode as Key;
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

        // Custom aliases used in config files to represent scancodes for disguised relative events.
        // Relative events are disguised into key events with those scancodes,
        // and are then sent through modmap and keymap.
        //
        // These custom aliases are used in config files, like other aliases.
        // The difference here is that since these scancodes don't map to any existing name,
        // (on purpose, to avoid conflating disguised events and actual key events)
        // we need to define them using scancodes instead of existing names.
        //
        // The DISGUISED_EVENT_OFFSETTER const is used here to make it easy to change the scancodes should it ever be necessary.
        // Because configs use name and custom aliases, changing their assigned value doesn't change how to write configs;
        // In other words, a config that works when DISGUISED_EVENT_OFFSETTER == 59974
        // will work exactly the same way if DISGUISED_EVENT_OFFSETTER == 46221
        //
        // DISGUISED_EVENT_OFFSETTER is also used in tests.rs::verify_disguised_relative_events(),
        // to prevent its modification to a number too low or too big.
        //
        // Cursor movement
        "XRIGHTCURSOR" => Key(DISGUISED_EVENT_OFFSETTER), // Cursor right
        "XLEFTCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 1), // Cursor left
        "XDOWNCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 2), // Cursor down
        "XUPCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 3), // Cursor up
        // Cursor... forward and backwards?
        "XREL_Z_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 4),
        "XREL_Z_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 5),
        //
        // Rotative cursor movement?
        "XREL_RX_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 6), // horizontal
        "XREL_RX_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 7),
        "XREL_RY_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 8), // vertical
        "XREL_RY_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 9),
        "XREL_RZ_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 10), // Whatever the third dimensional axis is called
        "XREL_RZ_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 11),
        //
        "XRIGHTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 12), // Rightscroll
        "XLEFTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 13),  // Leftscroll
        //
        // ???
        "XREL_DIAL_1" => Key(DISGUISED_EVENT_OFFSETTER + 14),
        "XREL_DIAL_2" => Key(DISGUISED_EVENT_OFFSETTER + 15),
        //
        "XUPSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 16), // Upscroll
        "XDOWNSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 17), // Downscroll
        //
        // Something?
        "XREL_MISC_1" => Key(DISGUISED_EVENT_OFFSETTER + 18),
        "XREL_MISC_2" => Key(DISGUISED_EVENT_OFFSETTER + 19),
        "XREL_RESERVED_1" => Key(DISGUISED_EVENT_OFFSETTER + 20),
        "XREL_RESERVED_2" => Key(DISGUISED_EVENT_OFFSETTER + 21),
        //
        // High resolution version of scroll events, sent just after their non-high resolution version.
        "XHIRES_UPSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 22),
        "XHIRES_DOWNSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 23),
        "XHIRES_RIGHTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 24),
        "XHIRES_LEFTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 25),
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
        "ANY" => KEY_MATCH_ANY,
        // End of custom scancodes

        // else
        _ => Key::KEY_RESERVED,
    };
    if key != Key::KEY_RESERVED {
        return Ok(key);
    }

    return Err(format!("unknown key '{}'", input).into());
}
