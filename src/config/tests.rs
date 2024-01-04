use crate::Config;
use indoc::indoc;
use serde_yaml::Error;

#[test]
fn test_modmap_basic() {
    assert_parse(indoc! {"
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
fn test_modmap_application() {
    assert_parse(indoc! {"
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
fn test_modmap_application_regex() {
    assert_parse(indoc! {r"
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
fn test_modmap_multi_purpose_key() {
    assert_parse(indoc! {"
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
fn test_modmap_multi_purpose_key_multi_key() {
    assert_parse(indoc! {"
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
fn test_virtual_modifiers() {
    assert_parse(indoc! {"
    virtual_modifiers:
      - CapsLock
    "})
}

#[test]
fn test_modmap_press_release_key() {
    assert_parse(indoc! {r#"
    modmap:
      - remap:
          Space:
            press: { launch: ["wmctrl", "-x", "-a", "code.Code"] }
            release: { launch: ["wmctrl", "-x", "-a", "nocturn.Nocturn"] }
    "#})
}

#[test]
fn test_keymap_basic() {
    assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt-Enter: Ctrl-Enter
      - remap:
          M-S: C-S
    "})
}

#[test]
fn test_keymap_lr_modifiers() {
    assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt_L-Enter: Ctrl_L-Enter
      - remap:
          M_R-S: C_L-S
    "})
}

#[test]
fn test_keymap_application() {
    assert_parse(indoc! {"
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
fn test_keymap_array() {
    assert_parse(indoc! {"
    keymap:
      - remap:
          C-w:
            - Shift-C-w
            - C-x
    "})
}

#[test]
fn test_keymap_remap() {
    assert_parse(indoc! {"
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
fn test_keymap_launch() {
    assert_parse(indoc! {r#"
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
fn test_keymap_mode() {
    assert_parse(indoc! {"
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
fn test_keymap_mark() {
    assert_parse(indoc! {"
    keymap:
      - remap:
          C-space: { set_mark: true }
          C-g: [esc, { set_mark: false }]
          C-b: { with_mark: left }
          M-b: { with_mark: C-left }
    "})
}

#[test]
fn test_shared_data_anchor() {
    assert_parse(indoc! {"
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
fn test_fail_on_data_outside_of_config_model() {
    assert_parse(indoc! {"
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

fn assert_parse(yaml: &str) {
    let result: Result<Config, Error> = serde_yaml::from_str(yaml);
    if let Err(e) = result {
        panic!("{}", e)
    }
}
