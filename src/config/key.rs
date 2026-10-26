use crate::config::key_aliases::lookup_key_alias;
use crate::config::key_combo::parse_modifier_alias;
use crate::event_handler::{DISGUISED_EVENT_OFFSETTER, KEY_MATCH_ANY};
use anyhow::Context;
use evdev::KeyCode as Key;
use std::error::Error;
use std::str::FromStr;

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

    // Replace custom keys defined by the user with evdev keys.
    // It's tested first, so the custom keys overwrite evdev keys.
    // So it wont break users' config, if evdev adds more keys.
    if let Some(key) = lookup_key_alias(&name) {
        return Ok(key);
    }

    // Original evdev scancodes should always work
    if let Ok(key) = Key::from_str(&name) {
        return Ok(key);
    }

    // You can abbreviate "KEY_" of any "KEY_*" scancodes.
    if let Ok(key) = Key::from_str(&format!("KEY_{name}")) {
        return Ok(key);
    }

    // By key code, (e.g. CODE_123)
    if name.starts_with("CODE_") {
        let key_code = name.replacen("CODE_", "", 1);
        let key_code = u16::from_str_radix(&key_code, 10).context(format!("Invalid key_code: {name}"))?;
        return Ok(Key(key_code));
    }

    // xremap's custom aliases like k0kubun/karabiner-dsl
    match &name[..] {
        // Shift
        "SHIFT_R" => Ok(Key::KEY_RIGHTSHIFT),
        "SHIFT_L" => Ok(Key::KEY_LEFTSHIFT),
        "S_R" => Ok(Key::KEY_RIGHTSHIFT),
        "S_L" => Ok(Key::KEY_LEFTSHIFT),
        // Control
        "CONTROL_R" => Ok(Key::KEY_RIGHTCTRL),
        "CONTROL_L" => Ok(Key::KEY_LEFTCTRL),
        "CTRL_R" => Ok(Key::KEY_RIGHTCTRL),
        "CTRL_L" => Ok(Key::KEY_LEFTCTRL),
        "C_R" => Ok(Key::KEY_RIGHTCTRL),
        "C_L" => Ok(Key::KEY_LEFTCTRL),
        // Alt
        "ALT_R" => Ok(Key::KEY_RIGHTALT),
        "ALT_L" => Ok(Key::KEY_LEFTALT),
        "A_R" => Ok(Key::KEY_RIGHTALT),
        "A_L" => Ok(Key::KEY_LEFTALT),
        "M_R" => Ok(Key::KEY_RIGHTALT),
        "M_L" => Ok(Key::KEY_LEFTALT),
        // Windows
        "SUPER_R" => Ok(Key::KEY_RIGHTMETA),
        "SUPER_L" => Ok(Key::KEY_LEFTMETA),
        "WINDOWS_R" => Ok(Key::KEY_RIGHTMETA),
        "WINDOWS_L" => Ok(Key::KEY_LEFTMETA),
        "WIN_R" => Ok(Key::KEY_RIGHTMETA),
        "WIN_L" => Ok(Key::KEY_LEFTMETA),
        "W_R" => Ok(Key::KEY_RIGHTMETA),
        "W_L" => Ok(Key::KEY_LEFTMETA),

        // Pseudo keys for evdev relative events.
        // REL_X
        "XRIGHTCURSOR" => Ok(Key(DISGUISED_EVENT_OFFSETTER)),
        "XLEFTCURSOR" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 1)),
        // REL_Y
        "XDOWNCURSOR" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 2)),
        "XUPCURSOR" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 3)),
        // REL_Z
        "XREL_Z_AXIS_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 4)),
        "XREL_Z_AXIS_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 5)),
        // REL_RX
        "XREL_RX_AXIS_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 6)),
        "XREL_RX_AXIS_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 7)),
        // REL_RY
        "XREL_RY_AXIS_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 8)),
        "XREL_RY_AXIS_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 9)),
        // REL_RZ
        "XREL_RZ_AXIS_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 10)),
        "XREL_RZ_AXIS_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 11)),
        // REL_HWHEEL
        "XRIGHTSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 12)),
        "XLEFTSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 13)),
        // REL_DIAL
        "XREL_DIAL_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 14)),
        "XREL_DIAL_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 15)),
        // REL_WHEEL
        "XUPSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 16)),
        "XDOWNSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 17)),
        // REL_MISC
        "XREL_MISC_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 18)),
        "XREL_MISC_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 19)),
        // REL_RESERVED
        "XREL_RESERVED_1" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 20)),
        "XREL_RESERVED_2" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 21)),
        // REL_WHEEL_HI_RES
        "XHIRES_UPSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 22)),
        "XHIRES_DOWNSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 23)),
        // REL_HWHEEL_HI_RES
        "XHIRES_RIGHTSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 24)),
        "XHIRES_LEFTSCROLL" => Ok(Key(DISGUISED_EVENT_OFFSETTER + 25)),

        // Any key
        "ANY" => Ok(KEY_MATCH_ANY),

        _ => {
            // Give warning if it's nearly correct
            if parse_modifier_alias(input).is_some() {
                Err(format!("Modifiers must have left/right specified when used as key: '{input}'").into())
            } else {
                Err(format!("Unknown key '{input}'").into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tests::assert_invalid_config;
    use crate::tests::parse_config_for_test;
    use evdev::KeyCode;
    use indoc::indoc;

    #[test]
    fn test_yaml_parse_literal_key_code() {
        let config = parse_config_for_test(indoc! {"
            keymap:
              - remap:
                  Code_123: A
                  CODE_0012: A
            "});
        assert!(config.keymap_table.contains_key(&KeyCode(123)));
        assert!(config.keymap_table.contains_key(&KeyCode(12)));
    }

    #[test]
    fn test_invalid_literal_key_code() {
        assert_invalid_config(
            indoc! {"
                keymap:
                  - remap:
                      Code_abc: A
            "},
            "keymap[0].remap: Invalid key_code: CODE_ABC at line 3 column 7",
        );

        assert_invalid_config(
            indoc! {"
                keymap:
                  - remap:
                      Code_70000: A
            "},
            "keymap[0].remap: Invalid key_code: CODE_70000 at line 3 column 7",
        );
    }
}
