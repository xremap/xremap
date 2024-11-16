use evdev::Key;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
  /// Map the evdev keys to labels on the keyboard.
  ///  - It's the same mapping the OS does to change keyboard layouts. Xremap also
  ///      needs to do it, so config files can use the keyboard layout.
  ///  - It's possible to use pseudo keys (only xremap knows of), because they're reversed before emitting.
  ///  - The evdev keys are defined here: https://github.com/emberian/evdev/blob/master/src/scancodes.rs
  ///     - They can also be obtained by running `RUST_LOG=debug xremap config.yml`
  ///     - They are close to the QWERTY keyboard layout: https://en.wikipedia.org/wiki/QWERTY
  ///  - Keys, that map to the same key on your keyboard, can be omitted.
  pub static ref keyboard_layout_definition: HashMap<u16, &'static str> = HashMap::from([
      //for AZERTY - only partial layout
      // (Key::KEY_Q.code(), "A"),
      // (Key::KEY_A.code(), "Q"),
      // (Key::KEY_W.code(), "Z"),
      // (Key::KEY_Z.code(), "W"),

      //for QWERTY danish layout
      (Key::KEY_MINUS.code(), "+"),
      (Key::KEY_EQUAL.code(), "´"),
      (Key::KEY_LEFTBRACE.code(), "Å"),
      (Key::KEY_RIGHTBRACE.code(), "¨"),
      (Key::KEY_SEMICOLON.code(), "Æ"),
      (Key::KEY_APOSTROPHE.code(), "Ø"),
      (Key::KEY_BACKSLASH.code(), "'"),
      (Key::KEY_SLASH.code(), "dash"), // Can't use the pseudokey: `-`, because it would be a syntax error in config file.
      (Key::KEY_102ND.code(), "<"),
  ]);

  // Map from evdev key to pseudo_key_code
  pub static ref keyboard_layout: HashMap<u16, u16> = keyboard_layout_definition.clone().into_iter().map(|(a, _)| (a, 40000 + a)).collect();

  // Map from pseudo_key_code to evdev key
  pub static ref reverse_keyboard_layout: HashMap<u16, u16> = keyboard_layout.iter().map(|(&a, &b)| (b, a)).collect();

  // Map from user-defined key to pseudo_key_code
  pub static ref keyboard_layout_keys: HashMap<String, u16> = keyboard_layout_definition.clone().into_iter().map(|(a, b)| (b.to_ascii_uppercase(), 40000 + a)).collect();

}

///
/// - The key_name is what ever the user wants. It doesn't have to be related to the evdev key names.
///
pub fn apply_keyboard_layout(key_name: &str) -> Option<Key> {
    let key_name = key_name.to_uppercase();

    keyboard_layout_keys
        .get(&format!("{}", key_name))
        .or_else(|| keyboard_layout_keys.get(&format!("KEY_{}", key_name)))
        .map(|&code| Key(code))
}
