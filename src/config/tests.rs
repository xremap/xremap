use crate::Config;
use indoc::indoc;

extern crate serde_yaml;
extern crate toml;

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
            held: Shift_L
            alone: Space
      - remap:
          Muhenkan:
            held: Alt_L
            alone: Muhenkan
            alone_timeout_millis: 500
    "})
}
#[test]
fn test_yaml_modmap_multi_purpose_key_without_timeout() {
    yaml_assert_parse(indoc! {"
    modmap:
      - remap:
          Space:
            held: Shift_L
            alone: Space
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
            held: [Shift_L]
            alone: [Shift_L,A]
      - remap:
          Muhenkan:
            held: [Alt_L,Shift_L]
            alone: [Muhenkan]
            alone_timeout_millis: 500
    "})
}
#[test]
fn test_yaml_virtual_modifiers() {
    yaml_assert_parse(indoc! {"
    virtual_modifiers:
      - CapsLock
    "})
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
#[should_panic]
fn test_yaml_fail_on_data_outside_of_config_model() {
    yaml_assert_parse(indoc! {"
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
    held = [ \"Shift_L\" ]
    alone = \"Space\"

    [[modmap]]
    [modmap.remap.Muhenkan]
    held = [ \"Alt_L\", \"Shift_L\" ]
    alone = [ \"Muhenkan\" ]
    alone_timeout_millis = 500
    "})
}

#[test]
fn test_toml_modmap_multi_purpose_key_multi_key() {
    toml_assert_parse(indoc! {"
    [[modmap]]
    [modmap.remap.Space]
    held = [ \"Shift_L\" ]
    alone = [ \"Shift_L\", \"A\" ]

    [[modmap]]
    [modmap.remap.Muhenkan]
    held = [ \"Alt_L\", \"Shift_L\" ]
    alone = [ \"Muhenkan\" ]
    alone_timeout_millis = 500
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
#[should_panic]
fn test_toml_fail_on_data_outside_of_config_model() {
    toml_assert_parse(indoc! {"
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
