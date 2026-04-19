use crate::config::key_press::parse_modifier_alias;
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

// Correspondence between pseudo keys created by xremap and evdev relative events
// Alias for pseudo key is disguised relative event.
//
//    evdev event code  | xremap pseudo key   | xremap pseudo key
//                      | when positive value | when negative value
//                      |                     |
//    REL_X             | XRightCursor        | XLeftCursor
//    REL_Y             | XDownCursor         | XUpCursor
//    REL_Z             | XREL_Z_AXIS_1       | XREL_Z_AXIS_2
//    REL_RX            | XREL_RX_AXIS_1      | XREL_RX_AXIS_2
//    REL_RY            | XREL_RY_AXIS_1      | XREL_RY_AXIS_2
//    REL_RZ            | XREL_RZ_AXIS_1      | XREL_RZ_AXIS_2
//    REL_HWHEEL        | XRightScroll        | XLeftScroll
//    REL_DIAL          | XREL_DIAL_1         | XREL_DIAL_2
//    REL_WHEEL         | XUpScroll           | XDownScroll
//    REL_MISC          | XREL_MISC_1         | XREL_MISC_2
//    REL_RESERVED      | XREL_RESERVED_1     | XREL_RESERVED_2
//    REL_WHEEL_HI_RES  | XHIRES_UPSCROLL     | XHIRES_DOWNSCROLL
//    REL_HWHEEL_HI_RES | XHIRES_RIGHTSCROLL  | XHIRES_LEFTSCROLL
//
pub fn parse_key(input: &str) -> Result<Key, Box<dyn Error>> {
    // Everything is case-insensitive
    let name = input.to_uppercase();

    // Original evdev scancodes should always work
    if let Ok(key) = Key::from_str(&name) {
        return Ok(key);
    }

    // You can abbreviate "KEY_" of any "KEY_*" scancodes.
    if let Ok(key) = Key::from_str(&format!("KEY_{name}")) {
        return Ok(key);
    }

    // xremap's custom aliases like k0kubun/karabiner-dsl
    let key = match &name[..] {
        // Shift
        "SHIFT_R" => Key::KEY_RIGHTSHIFT,
        "SHIFT_L" => Key::KEY_LEFTSHIFT,
        "S_R" => Key::KEY_RIGHTSHIFT,
        "S_L" => Key::KEY_LEFTSHIFT,
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
        "A_R" => Key::KEY_RIGHTALT,
        "A_L" => Key::KEY_LEFTALT,
        "M_R" => Key::KEY_RIGHTALT,
        "M_L" => Key::KEY_LEFTALT,
        // Windows
        "SUPER_R" => Key::KEY_RIGHTMETA,
        "SUPER_L" => Key::KEY_LEFTMETA,
        "WIN_R" => Key::KEY_RIGHTMETA,
        "WIN_L" => Key::KEY_LEFTMETA,
        "W_R" => Key::KEY_RIGHTMETA,
        "W_L" => Key::KEY_LEFTMETA,

        // Pseudo keys for evdev relative events.
        // REL_X
        "XRIGHTCURSOR" => Key(DISGUISED_EVENT_OFFSETTER),
        "XLEFTCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 1),
        // REL_Y
        "XDOWNCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 2),
        "XUPCURSOR" => Key(DISGUISED_EVENT_OFFSETTER + 3),
        // REL_Z
        "XREL_Z_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 4),
        "XREL_Z_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 5),
        // REL_RX
        "XREL_RX_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 6),
        "XREL_RX_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 7),
        // REL_RY
        "XREL_RY_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 8),
        "XREL_RY_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 9),
        // REL_RZ
        "XREL_RZ_AXIS_1" => Key(DISGUISED_EVENT_OFFSETTER + 10),
        "XREL_RZ_AXIS_2" => Key(DISGUISED_EVENT_OFFSETTER + 11),
        // REL_HWHEEL
        "XRIGHTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 12),
        "XLEFTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 13),
        // REL_DIAL
        "XREL_DIAL_1" => Key(DISGUISED_EVENT_OFFSETTER + 14),
        "XREL_DIAL_2" => Key(DISGUISED_EVENT_OFFSETTER + 15),
        // REL_WHEEL
        "XUPSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 16),
        "XDOWNSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 17),
        // REL_MISC
        "XREL_MISC_1" => Key(DISGUISED_EVENT_OFFSETTER + 18),
        "XREL_MISC_2" => Key(DISGUISED_EVENT_OFFSETTER + 19),
        // REL_RESERVED
        "XREL_RESERVED_1" => Key(DISGUISED_EVENT_OFFSETTER + 20),
        "XREL_RESERVED_2" => Key(DISGUISED_EVENT_OFFSETTER + 21),
        // REL_WHEEL_HI_RES
        "XHIRES_UPSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 22),
        "XHIRES_DOWNSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 23),
        // REL_HWHEEL_HI_RES
        "XHIRES_RIGHTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 24),
        "XHIRES_LEFTSCROLL" => Key(DISGUISED_EVENT_OFFSETTER + 25),

        // Any key
        "ANY" => KEY_MATCH_ANY,
        // End of custom scancodes

        // else
        _ => Key::KEY_RESERVED,
    };
    if key != Key::KEY_RESERVED {
        return Ok(key);
    }

    // Give warning if it's nearly correct
    if parse_modifier_alias(input).is_some() {
        return Err(format!("Modifiers must have left/right specified when used as key: '{input}'").into());
    }

    Err(format!("unknown key '{input}'").into())
}

#[test]
fn test_parse_key() {
    // Can omit the 'KEY_' prefex
    assert_eq!(parse_key("Enter").unwrap(), Key::KEY_ENTER);

    // Can use lower case
    assert_eq!(parse_key("key_enter").unwrap(), Key::KEY_ENTER);

    // Can use uppercase keys
    assert_eq!(parse_key("KEY_ENTER").unwrap(), Key::KEY_ENTER);

    // S is KEY_S, not shift.
    assert_eq!(parse_key("S").unwrap(), Key::KEY_S);

    // Modifier without sidedness can't be a key.
    assert_eq!(
        parse_key("Shift").unwrap_err().to_string(),
        "Modifiers must have left/right specified when used as key: 'Shift'"
    );
}
