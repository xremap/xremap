use crate::Config;
use indoc::indoc;

#[test]
fn test_yaml_modmap_basic() {
    yaml_assert_parse(indoc! {"
    modmap:
      - name: Global
        remap:
          Alt_L: Ctrl_L
      - remap:
          Shift_R: Win_R
        application:
          only: Google-chrome
    "})
}

#[test]
fn test_yaml_modmap_application() {
    yaml_assert_parse(indoc! {"
    modmap:
      - remap:
          Alt_L: Ctrl_L
        application:
          not:
            - Gnome-terminal
      - remap:
          Shift_R: Win_R
        application:
          only: Google-chrome
    "})
}

#[test]
fn test_yaml_modmap_application_regex() {
    yaml_assert_parse(indoc! {r"
    modmap:
      - remap:
          Alt_L: Ctrl_L
        application:
          not:
            - /^Minecraft/
            - /^Minecraft\//
            - /^Minecraft\d/
      - remap:
          Shift_R: Win_R
        application:
          only: /^Miencraft\\/
    "})
}

#[test]
fn test_yaml_modmap_multi_purpose_key() {
    yaml_assert_parse(indoc! {"
    modmap:
      - remap:
          Space:
            hold: Shift_L
            tap: Space
      - remap:
          Muhenkan:
            hold: Alt_L
            tap: Muhenkan
            tap_timeout_millis: 500
    "})
}
#[test]
fn test_yaml_modmap_multi_purpose_key_without_timeout() {
    yaml_assert_parse(indoc! {"
    modmap:
      - remap:
          Space:
            hold: Shift_L
            tap: Space
            free_hold: true
    "})
    // NOTE: add edge cases tests for when timeout = default
}

#[test]
fn test_yaml_modmap_multi_purpose_key_multi_key() {
    yaml_assert_parse(indoc! {"
    modmap:
      - remap:
          Space:
            hold: [Shift_L]
            tap: [Shift_L,A]
      - remap:
          Muhenkan:
            hold: [Alt_L,Shift_L]
            tap: [Muhenkan]
            tap_timeout_millis: 500
    "})
}

#[test]
fn test_yaml_modmap_multi_purpose_key_with_invalid_key() {
    // This fails silently.
    let _errmsg = serde_yaml::from_str::<Config>(indoc! {"
    modmap:
      - remap:
          Space:
            alone: A
            held: SomeCustomKey
        "
    });
}

#[test]
fn test_yaml_virtual_modifiers() {
    yaml_assert_parse(indoc! {"
    virtual_modifiers:
      - CapsLock
    "})
}

#[test]
fn test_yaml_fail_on_modifier_missing_sidedness() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        modmap:
          - remap:
              WIN: Control_L
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(&errmsg, "modmap[0].remap: unknown key 'WIN' at line 3 column 7");
}

#[test]
fn test_yaml_fail_on_modifier_in_modmap_match() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        modmap:
          - remap:
              C-COMMA: Control_L-C
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(&errmsg, "modmap[0].remap: unknown key 'C-COMMA' at line 3 column 7");
}

#[test]
fn test_yaml_invalid_virtual_modifiers_1() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        virtual_modifiers:
            - WIN
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(&errmsg, "unknown key 'WIN'");
}

#[test]
fn test_yaml_invalid_virtual_modifiers_2() {
    // Fails silently
    serde_yaml::from_str::<Config>(indoc! {"
        virtual_modifiers:
            - WIN_L
        "
    })
    .unwrap();
}

#[test]
fn test_yaml_modmap_press_release_key() {
    yaml_assert_parse(indoc! {r#"
    modmap:
      - remap:
          Space:
            press: { launch: ["wmctrl", "-x", "-a", "code.Code"] }
            release: { launch: ["wmctrl", "-x", "-a", "nocturn.Nocturn"] }
    "#})
}

#[test]
fn test_yaml_press_release_key_wrongly_used_in_keymap() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        keymap:
          - remap:
              CapsLock:
                press: esc
                release: esc
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 7"
    );
}

#[test]
fn test_yaml_modifier_is_missing_sidedness() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        keymap:
          - remap:
              CapsLock: ctrl
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 7"
    );
}

#[test]
fn test_yaml_key_is_unknown() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
        keymap:
          - remap:
              CapsLock: escape
        "
    })
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 7"
    );
}

#[test]
fn test_yaml_keymap_basic() {
    yaml_assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt-Enter: Ctrl-Enter
      - remap:
          M-S: C-S
    "})
}

#[test]
fn test_yaml_keymap_lr_modifiers() {
    yaml_assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt_L-Enter: Ctrl_L-Enter
      - remap:
          M_R-S: C_L-S
    "})
}

#[test]
fn test_yaml_keymap_application() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          Alt-Enter: Ctrl-Enter
        application:
          not: Gnome-terminal
      - remap:
          Alt-S: Ctrl-S
        application:
          only:
            - Gnome-terminal
    "})
}

#[test]
fn test_yaml_keymap_array() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          C-w:
            - Shift-C-w
            - C-x
    "})
}

#[test]
fn test_yaml_keymap_remap() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          C-x:
            remap:
              s: C-w
              C-s:
                remap:
                  x: C-z
            timeout_key: Down
            timeout_millis: 1000
    "})
}

#[test]
fn test_yaml_keymap_remap_timeout_as_sequence() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          C-x:
            remap:
              s: C-w
              C-s:
                remap:
                  x: C-z
            timeout_key: [Down,Up]
            timeout_millis: 1000
    "})
}

#[test]
fn test_yaml_keymap_remap_timeout_key_invalid() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
    keymap:
      - remap:
          C-x:
            timeout_key: invalid_key
            remap:
              s: C-w
    "})
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 7"
    );
}

#[test]
fn test_yaml_keymap_remap_timeout_invalid() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
    keymap:
      - remap:
          C-x:
            timeout_millis: k
            remap:
              s: C-w
    "})
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 7"
    );
}

#[test]
fn test_yaml_keymap_remap_property_invalid() {
    // Fails silently
    serde_yaml::from_str::<Config>(indoc! {"
    keymap:
      - remap:
          C-x:
            not_valid_property: k
            remap:
              s: C-w
    "})
    .ok();
}

#[test]
fn test_yaml_keymap_launch() {
    yaml_assert_parse(indoc! {r#"
    keymap:
      - remap:
          KEY_GRAVE:
            launch:
              - "/bin/sh"
              - "-c"
              - "date > /tmp/hotkey_test"
    "#})
}

#[test]
fn test_yaml_keymap_mode() {
    yaml_assert_parse(indoc! {"
    default_mode: insert
    keymap:
      - mode: insert
        remap:
          Esc: { set_mode: normal }
      - mode: normal
        remap:
          i: { set_mode: insert }
          h: Left
          j: Down
          k: Up
          l: Right
    "})
}

#[test]
fn test_yaml_keymap_mark() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          C-space: { set_mark: true }
          C-g: [esc, { set_mark: false }]
          C-b: { with_mark: left }
          M-b: { with_mark: C-left }
    "})
}

#[test]
fn test_yaml_shared_data_anchor() {
    yaml_assert_parse(indoc! {"
    shared:
      terminals: &terminals
        - Gnome-terminal
        - Kitty

    modmap:
      - remap:
          Alt_L: Ctrl_L
        application:
          not: *terminals
      - remap:
          Shift_R: Win_R
        application:
          only: Google-chrome
    "})
}

#[test]
fn test_yaml_fail_on_data_outside_of_config_model() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
    terminals: &terminals
      - Gnome-terminal
      - Kitty

    modmap:
      - remap:
          Alt_L: Ctrl_L
        application:
          not: *terminals
      - remap:
          Shift_R: Win_R
        application:
          only: Google-chrome
    "})
    .unwrap_err()
    .to_string();

    assert!(errmsg.contains("unknown field `terminals`"));
}

#[test]
fn test_yaml_no_keymap_action() {
    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          f12: []
    "});

    yaml_assert_parse(indoc! {"
    keymap:
      - remap:
          f12: null
    "})
}

#[test]
fn test_toml_modmap_basic() {
    toml_assert_parse(indoc! {"
    [[modmap]]
    name = \"Global\"
    [modmap.remap]
    Alt_L = \"Ctrl_L\"

    [[modmap]]
    [modmap.remap]
    Shift_R = \"Win_R\"

    [modmap.application]
    only = \"Google-chrome\"
    "})
}

#[test]
fn test_toml_modmap_application() {
    toml_assert_parse(indoc! {"
    [[modmap]]
    [modmap.remap]
    Alt_L = \"Ctrl_L\"

    [modmap.application]
    not = [ \"Gnome-terminal\" ]

    [[modmap]]
    [modmap.remap]
    Shift_R = \"Win_R\"

    [modmap.application]
    only = \"Google-chrome\"

    "})
}

#[test]
fn test_toml_modmap_application_regex() {
    toml_assert_parse(indoc! {r#"
    [[modmap]]
    [modmap.remap]
    Alt_L = "Ctrl_L"

    [modmap.application]
    not = [ "/^Minecraft/", "/^Minecraft\\//", "/^Minecraft\\d/" ]

    [[modmap]]
    [modmap.remap]
    Shift_R = "Win_R"

    [modmap.application]
    only = "/^Miencraft\\\\/"

    "#})
}

#[test]
fn test_toml_modmap_multi_purpose_key() {
    toml_assert_parse(indoc! {"
    [[modmap]]
    [modmap.remap.Space]
    hold = [ \"Shift_L\" ]
    tap = \"Space\"

    [[modmap]]
    [modmap.remap.Muhenkan]
    hold = [ \"Alt_L\", \"Shift_L\" ]
    tap = [ \"Muhenkan\" ]
    tap_timeout_millis = 500
    "})
}

#[test]
fn test_toml_modmap_multi_purpose_key_multi_key() {
    toml_assert_parse(indoc! {"
    [[modmap]]
    [modmap.remap.Space]
    hold = [ \"Shift_L\" ]
    tap = [ \"Shift_L\", \"A\" ]

    [[modmap]]
    [modmap.remap.Muhenkan]
    hold = [ \"Alt_L\", \"Shift_L\" ]
    tap = [ \"Muhenkan\" ]
    tap_timeout_millis = 500
    "})
}
#[test]
fn test_toml_virtual_modifiers() {
    toml_assert_parse(indoc! {"
    virtual_modifiers = [ \"CapsLock\" ]
    "})
}

#[test]
fn test_toml_modmap_press_release_key() {
    toml_assert_parse(indoc! {r#"
    [[modmap]]
    [modmap.remap.Space.press]
    launch = [ "wmctrl", "-x", "-a", "code.Code" ]
    [modmap.remap.Space.release]
    launch = ["wmctrl", "-x", "-a", "nocturn.Nocturn"]
    "#})
}

#[test]
fn test_toml_keymap_basic() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    name = \"Global\"
    [keymap.remap]
    Alt-Enter = \"Ctrl-Enter\"

    [[keymap]]
    [keymap.remap]
    M-S = \"C-S\"
    "})
}

#[test]
fn test_toml_keymap_lr_modifiers() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    name = \"Global\"
    [keymap.remap]
    Alt_L-Enter = \"Ctrl_L-Enter\"

    [[keymap]]
    [keymap.remap]
    M_R-S = \"C_L-S\"
    "})
}

#[test]
fn test_toml_keymap_application() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap]
    Alt-Enter = \"Ctrl-Enter\"
    [keymap.application]
    not = \"Gnome-terminal\"
    [[keymap]]
    [keymap.remap]
    Alt-S = \"Ctrl-S\"
    [keymap.application]
    only = \"Gnome-terminal\"
    "})
}

#[test]
fn test_toml_keymap_array() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap]
    C-w = [\"Shift-C-w\", \"C-x\"]
    "})
}

#[test]
fn test_toml_keymap_remap() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap.C-x]
    timeout_key = \"Down\"
    timeout_millis = 1_000

    [keymap.remap.C-x.remap]
    s = \"C-w\"

    [keymap.remap.C-x.remap.C-s.remap]
    x = \"C-z\"
    "})
}
#[test]
fn test_toml_keymap_remap_timeout_key_sequence() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap.C-x]
    timeout_key = [ \"Down\", \"UP\" ]
    timeout_millis = 1_000

    [keymap.remap.C-x.remap]
    s = \"C-w\"

    [keymap.remap.C-x.remap.C-s.remap]
    x = \"C-z\"
    "})
}

#[test]
fn test_toml_keymap_launch() {
    toml_assert_parse(indoc! {r#"
    [[keymap]]
    [keymap.remap.KEY_GRAVE]
    launch = [ "/bin/sh", "-c", "date > /tmp/hotkey_test" ]
    "#})
}

#[test]
fn test_toml_keymap_mode() {
    toml_assert_parse(indoc! {"
    default_mode = \"insert\"

    [[keymap]]
    mode = \"instert\"

    [keymap.remap.Esc]
    set_mode = \"normal\"

    [[keymap]]
    mode = \"normal\"

    [keymap.remap]
    h = \"Left\"
    j = \"Down\"
    k = \"Up\"
    l = \"Right\"

    [keymap.remap.i]
    set_mode = \"insert\"

    "})
}

#[test]
fn test_toml_keymap_mark() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap]
    C-g = [ \"esc\", { set_mark = false } ]

    [keymap.remap.C-space]
    set_mark = true

    [keymap.remap.C-b]
    with_mark = \"left\"

    [keymap.remap.M-b]
    with_mark = \"C-left\"
    "})
}

#[test]
fn test_toml_shared_data_anchor() {
    toml_assert_parse(indoc! {"
    [shared]
    terminals = [ \"Gnome-terminal\", \"Kitty\" ]

    [[shared.modmap]]
    [shared.modmap.remap]
    Alt_L = \"Ctrl_L\"

    [shared.modmap.application]
    not = \"terminals\"

    [[shared.modmap]]
    [shared.modmap.remap]
    Shift_R = \"Win_R\"

    [shared.modmap.application]
    only = \"Google-chrome\"
    "})
}

#[test]
fn test_toml_fail_on_data_outside_of_config_model() {
    let errmsg = toml::from_str::<Config>(indoc! {"
    terminals = [ \"Gnome-terminal\", \"Kitty\" ]

    [[modmap]]
    [modmap.remap]
    Alt_L = \"Ctrl_L\"

    [modmap.application]
    not = [ \"Gnome-terminal\", \"Kitty\" ]

    [[modmap]]
    [modmap.remap]
    Shift_R = \"Win_R\"

    [modmap.application]
    only = \"Google-chrome\"
    "})
    .unwrap_err()
    .to_string();

    assert!(errmsg.contains("unknown field `terminals`"));
}

#[test]
fn test_toml_no_keymap_action() {
    toml_assert_parse(indoc! {"
    [[keymap]]
    [keymap.remap]
    f12 = []
    "})
}

fn toml_assert_parse(toml: &str) {
    let result: Result<Config, toml::de::Error> = toml::from_str(toml);
    if let Err(e) = result {
        panic!("{}", e)
    }
}

fn yaml_assert_parse(yaml: &str) {
    let result: Result<Config, serde_yaml::Error> = serde_yaml::from_str(yaml);
    if let Err(e) = result {
        panic!("{}", e)
    }
}
