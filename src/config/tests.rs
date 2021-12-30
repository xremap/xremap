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
fn test_keymap_basic() {
    assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt-Enter: Ctrl-Enter
      - remap:
          Alt-S: Ctrl-S
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
    "})
}

#[test]
fn test_keymap_launch() {
    assert_parse(indoc! {"
    keymap:
      - remap:
          KEY_GRAVE:
            launch:
              - '/bin/sh'
              - '-c'
              - 'date > /tmp/hotkey_test'
    "})
}

fn assert_parse(yaml: &str) {
    let result: Result<Config, Error> = serde_yaml::from_str(&yaml);
    if let Err(e) = result {
        assert!(false, "{}", e)
    }
}
