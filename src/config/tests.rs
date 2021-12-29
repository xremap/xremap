use crate::Config;
use indoc::indoc;
use serde_yaml::Error;

#[test]
fn test_basic_remap() {
    assert_parse(indoc! {"
    keymap:
      - name: Global
        remap:
          Alt-Enter: Ctrl-Enter
    "})
}

fn assert_parse(yaml: &str) {
    let result: Result<Config, Error> = serde_yaml::from_str(&yaml);
    if let Err(e) = result {
        assert!(false, "{}", e)
    }
}
