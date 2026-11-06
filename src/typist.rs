//! Text → key chords, using the actual keyboard layout.
//!
//! Resolving a character to a key requires the XKB layout: the same physical
//! key produces different characters per layout, and some characters need
//! Shift or AltGr. We compile an XKB keymap with libxkbcommon and pre-compute
//! a `char → (keycode, modifiers)` table once per (layout, variant).
//!
//! libxkbcommon is loaded at runtime (see `xkbcommon-dl`), so xremap keeps
//! building without any C dependency. When it is unavailable, `type` reports
//! an error instead of silently typing the wrong characters.

use evdev::KeyCode as Key;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use xkbcommon_dl::{
    xkb_compose_compile_flags, xkb_compose_state, xkb_compose_state_flags, xkb_compose_status,
    xkb_compose_table, xkb_context, xkb_context_flags, xkb_keymap, xkb_keymap_compile_flags,
    xkb_rule_names, xkb_state, xkbcommon_compose_option, xkbcommon_option, XkbCommonCompose,
};

/// How characters that the layout cannot produce are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMethod {
    /// Use the layout, and fall back to `Ctrl+Shift+U` for anything else.
    Auto,
    /// Only type what the layout produces; unsupported characters are an error.
    Layout,
    /// Always use the `Ctrl+Shift+U` Unicode input sequence.
    Unicode,
    /// Paste through the clipboard instead of typing. Handled outside the
    /// typist, since no layout is involved.
    Clipboard,
}

impl TextMethod {
    pub fn parse(word: &str) -> Option<TextMethod> {
        match word {
            "auto" => Some(TextMethod::Auto),
            "layout" => Some(TextMethod::Layout),
            "unicode" => Some(TextMethod::Unicode),
            "clipboard" => Some(TextMethod::Clipboard),
            _ => None,
        }
    }
}

/// A character that can be typed: the key to press and the modifiers to hold.
#[derive(Debug, Clone, PartialEq)]
pub struct CharChord {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

/// One (keysym, character, chord) combination the layout offers.
struct Candidate {
    keysym: u32,
    /// `None` for keys that produce no character on their own (dead keys).
    ch: Option<char>,
    chord: CharChord,
}

#[derive(PartialEq)]
enum ComposeStep {
    Other,
    Composing,
    Composed(char),
}

/// Open the Compose table for the current locale, falling back to the locales
/// that every desktop installs.
unsafe fn load_compose_table(
    xkb: &XkbCommonCompose,
    context: *mut xkb_context,
) -> *mut xkb_compose_table {
    let from_env = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    let mut locales: Vec<String> = from_env.into_iter().collect();
    locales.push("en_US.UTF-8".to_string());
    locales.push("C.UTF-8".to_string());
    for locale in locales {
        let Ok(locale) = CString::new(locale) else { continue };
        let table = (xkb.xkb_compose_table_new_from_locale)(
            context,
            locale.as_ptr() as *const c_char,
            xkb_compose_compile_flags::XKB_COMPOSE_COMPILE_NO_FLAGS,
        );
        if !table.is_null() {
            return table;
        }
    }
    ptr::null_mut()
}

/// Reset `state`, feed `keysyms`, and return the resulting compose step.
unsafe fn compose_eval(
    xkb: &XkbCommonCompose,
    state: *mut xkb_compose_state,
    keysyms: &[u32],
) -> ComposeStep {
    (xkb.xkb_compose_state_reset)(state);
    for keysym in keysyms {
        (xkb.xkb_compose_state_feed)(state, *keysym);
    }
    match (xkb.xkb_compose_state_get_status)(state) {
        xkb_compose_status::XKB_COMPOSE_COMPOSING => ComposeStep::Composing,
        xkb_compose_status::XKB_COMPOSE_COMPOSED => {
            let mut buffer = [0u8; 64];
            let required = (xkb.xkb_compose_state_get_utf8)(
                state, buffer.as_mut_ptr().cast::<c_char>(), buffer.len(),
            );
            // The API reports the required size, not the number actually
            // copied. Oversized/multicharacter results are not single-char
            // mappings and must not be entered into the typing table.
            composed_character(&buffer, required).map_or(ComposeStep::Other, ComposeStep::Composed)
        }
        _ => ComposeStep::Other,
    }
}

fn composed_character(buffer: &[u8], required: i32) -> Option<char> {
    let required = usize::try_from(required).ok()?;
    if required == 0 || required >= buffer.len() {
        return None;
    }
    let text = std::str::from_utf8(&buffer[..required]).ok()?;
    let mut chars = text.chars();
    let ch = chars.next()?;
    (chars.next().is_none() && !ch.is_control()).then_some(ch)
}

/// Characters typed as dedicated keys on every layout.
fn literal_key(ch: char) -> Option<Key> {
    match ch {
        '\n' => Some(Key::KEY_ENTER),
        '\t' => Some(Key::KEY_TAB),
        _ => None,
    }
}

/// Modifier keys for a libxkbcommon modifier mask.
#[derive(Clone, Copy, PartialEq, Default)]
struct Masks {
    shift: u32,
    altgr: u32,
    ctrl: u32,
    alt: u32,
}

pub struct Typist {
    /// Compiled per (layout, variant); `None` caches a failed compilation.
    keymaps: HashMap<(String, String), Option<Keymap>>,
}

struct Keymap {
    keymap: *mut xkb_keymap,
    state: *mut xkb_state,
    /// Characters produced by a single key press.
    chars: HashMap<char, CharChord>,
    /// Characters that need a dead-key sequence (`´` then `e` for `é`).
    sequences: HashMap<char, Vec<CharChord>>,
}

impl Drop for Keymap {
    fn drop(&mut self) {
        if let Some(xkb) = xkbcommon_option() {
            unsafe {
                (xkb.xkb_state_unref)(self.state);
                (xkb.xkb_keymap_unref)(self.keymap);
            }
        }
    }
}

impl Default for Typist {
    fn default() -> Self {
        Self::new()
    }
}

impl Typist {
    pub fn new() -> Self {
        Self { keymaps: HashMap::new() }
    }

    /// Turn `text` into a list of chords for the given layout.
    /// `layout` defaults to `$XKB_DEFAULT_LAYOUT`, then `"us"`, matching the
    /// environment xkbcommon itself uses.
    pub fn chords(
        &mut self,
        layout: Option<&str>,
        variant: Option<&str>,
        method: TextMethod,
        text: &str,
    ) -> Result<Vec<CharChord>, String> {
        let layout = layout
            .map(str::to_string)
            .or_else(|| std::env::var("XKB_DEFAULT_LAYOUT").ok().filter(|l| !l.is_empty()))
            .unwrap_or_else(|| "us".to_string());
        let variant = variant
            .map(str::to_string)
            .or_else(|| std::env::var("XKB_DEFAULT_VARIANT").ok())
            .unwrap_or_default();

        let keymap = self
            .keymap(&layout, &variant)?
            .ok_or_else(|| format!("cannot compile XKB keymap for layout '{layout}'"))?;

        let mut chords = Vec::with_capacity(text.chars().count());
        for ch in text.chars() {
            if let Some(key) = literal_key(ch) {
                chords.push(CharChord { modifiers: vec![], key });
                continue;
            }
            if method == TextMethod::Unicode {
                chords.extend(keymap.unicode_chord(ch).ok_or_else(|| unicode_error(ch))?);
                continue;
            }
            if let Some(chord) = keymap.chars.get(&ch) {
                chords.push(chord.clone());
                continue;
            }
            // A dead-key sequence: `´` then `e` types `é`.
            if let Some(sequence) = keymap.sequences.get(&ch) {
                chords.extend(sequence.iter().cloned());
                continue;
            }
            match method {
                // The layout cannot produce it: fall back to the X11 Unicode
                // input sequence, which GUI toolkits implement themselves.
                TextMethod::Auto => chords.extend(keymap.unicode_chord(ch).ok_or_else(|| unicode_error(ch))?),
                _ => {
                    return Err(format!(
                        "character {ch:?} is not available on layout '{layout}'{}",
                        if variant.is_empty() { String::new() } else { format!(" ({variant})") }
                    ))
                }
            }
        }
        Ok(chords)
    }

    fn keymap(&mut self, layout: &str, variant: &str) -> Result<Option<&Keymap>, String> {
        let key = (layout.to_string(), variant.to_string());
        if !self.keymaps.contains_key(&key) {
            let compiled = Keymap::compile(layout, variant);
            self.keymaps.insert(key.clone(), compiled);
        }
        Ok(self.keymaps.get(&key).and_then(|km| km.as_ref()))
    }
}

fn unicode_error(ch: char) -> String {
    format!("cannot type {ch:?}: the layout has no key for it and the Ctrl+Shift+U sequence is not available")
}

impl Keymap {
    /// `Ctrl+Shift+U`, the hex digits of the codepoint, then Enter.
    /// This is the X11/XKB Unicode input convention that GTK, Qt, Chromium and
    /// Electron-based applications implement.
    fn unicode_chord(&self, ch: char) -> Option<Vec<CharChord>> {
        let digits = format!("{:x}", ch as u32);
        let mut chords = Vec::with_capacity(digits.len() + 2);
        // Note: on layouts where `u` sits on the AltGr level this produces
        // Ctrl+Shift+AltGr+u, which some applications do not recognize. Use
        // `--method clipboard` there instead.
        let u = self.chars.get(&'u')?;
        let mut modifiers = u.modifiers.clone();
        for modifier in [Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT] {
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }
        chords.push(CharChord { modifiers, key: u.key });
        for digit in digits.chars() {
            chords.push(self.chars.get(&digit)?.clone());
        }
        chords.push(CharChord { modifiers: vec![], key: Key::KEY_ENTER });
        Some(chords)
    }

    fn compile(layout: &str, variant: &str) -> Option<Keymap> {
        let xkb = xkbcommon_option()?;
        let layout_c = CString::new(layout).ok()?;
        let variant_c = CString::new(variant).ok()?;
        unsafe {
            let context: *mut xkb_context =
                (xkb.xkb_context_new)(xkb_context_flags::XKB_CONTEXT_NO_FLAGS);
            if context.is_null() {
                return None;
            }
            let rules = xkb_rule_names {
                rules: ptr::null(),
                model: ptr::null(),
                layout: layout_c.as_ptr() as *const c_char,
                variant: if variant.is_empty() {
                    ptr::null()
                } else {
                    variant_c.as_ptr() as *const c_char
                },
                options: ptr::null(),
            };
            let keymap: *mut xkb_keymap = (xkb.xkb_keymap_new_from_names)(
                context,
                &rules,
                xkb_keymap_compile_flags::XKB_KEYMAP_COMPILE_NO_FLAGS,
            );
            if keymap.is_null() {
                (xkb.xkb_context_unref)(context);
                return None;
            }
            let state: *mut xkb_state = (xkb.xkb_state_new)(keymap);
            if state.is_null() {
                (xkb.xkb_keymap_unref)(keymap);
                (xkb.xkb_context_unref)(context);
                return None;
            }

            let mut masks = Masks::default();
            for (name, slot) in [
                ("Shift", &mut masks.shift),
                ("Mod5", &mut masks.altgr),
                ("Control", &mut masks.ctrl),
                ("Mod1", &mut masks.alt),
            ] {
                if let Ok(name) = CString::new(name) {
                    let index = (xkb.xkb_keymap_mod_get_index)(keymap, name.as_ptr());
                    if index != u32::MAX {
                        *slot = 1 << index;
                    }
                }
            }

            let candidates = Self::candidates(xkb, keymap, state, masks);
            let mut chars: HashMap<char, CharChord> = HashMap::new();
            for candidate in &candidates {
                if let Some(ch) = candidate.ch {
                    // The simplest combination wins: `a` stays unshifted even
                    // if a later keycode can also produce it.
                    chars.entry(ch).or_insert_with(|| candidate.chord.clone());
                }
            }
            let sequences = Self::build_sequences(context, &candidates);
            (xkb.xkb_context_unref)(context);
            Some(Keymap { keymap, state, chars, sequences })
        }
    }

    /// Walk every keycode and the modifier combinations we can synthesize,
    /// recording the keysym and the character each combination produces.
    /// Characters that need Ctrl or Alt are skipped: they are usually control
    /// sequences, not text.
    unsafe fn candidates(
        xkb: &xkbcommon_dl::XkbCommon,
        keymap: *mut xkb_keymap,
        state: *mut xkb_state,
        masks: Masks,
    ) -> Vec<Candidate> {
        // Only modifiers we can actually press are considered.
        let combinations: [(&[Key], u32); 4] = [
            (&[], 0),
            (&[Key::KEY_LEFTSHIFT], masks.shift),
            (&[Key::KEY_RIGHTALT], masks.altgr),
            (&[Key::KEY_LEFTSHIFT, Key::KEY_RIGHTALT], masks.shift | masks.altgr),
        ];
        let skip = masks.ctrl | masks.alt;

        let mut seen: HashMap<u32, usize> = HashMap::new();
        let mut candidates: Vec<Candidate> = Vec::new();
        let min = (xkb.xkb_keymap_min_keycode)(keymap);
        let max = (xkb.xkb_keymap_max_keycode)(keymap);
        for keycode in min..=max {
            for (modifier_keys, mask) in combinations {
                if mask & skip != 0 {
                    continue;
                }
                (xkb.xkb_state_update_mask)(state, mask, 0, 0, 0, 0, 0);
                let keysym = (xkb.xkb_state_key_get_one_sym)(state, keycode);
                if keysym == 0 {
                    continue;
                }
                let ch = match (xkb.xkb_state_key_get_utf32)(state, keycode) {
                    0 => None,
                    utf32 => char::from_u32(utf32).filter(|ch| !ch.is_control()),
                };
                let chord = CharChord {
                    modifiers: modifier_keys.to_vec(),
                    key: Key::new(keycode.saturating_sub(8) as u16),
                };
                // The same keysym can sit on several keys (and be reachable
                // with several modifier sets). Keep the first, simplest one.
                match seen.get(&keysym) {
                    Some(&index) => {
                        if candidates[index].ch.is_none() && ch.is_some() {
                            candidates[index].ch = ch;
                        }
                    }
                    None => {
                        seen.insert(keysym, candidates.len());
                        candidates.push(Candidate { keysym, ch, chord });
                    }
                }
            }
        }
        (xkb.xkb_state_update_mask)(state, 0, 0, 0, 0, 0, 0);
        candidates
    }

    /// Characters that no single key produces but that a dead-key sequence
    /// does, resolved through the system Compose table: `´` then `e` gives
    /// `é`, and three-part sequences such as `^` `´` `a` give `ấ`.
    unsafe fn build_sequences(
        context: *mut xkb_context,
        candidates: &[Candidate],
    ) -> HashMap<char, Vec<CharChord>> {
        let mut sequences: HashMap<char, Vec<CharChord>> = HashMap::new();
        // Dead keys need libxkbcommon's compose module; without it, sequences
        // are simply unavailable and `type` falls back as before.
        let Some(xkb) = xkbcommon_compose_option() else { return sequences };
        let table = load_compose_table(xkb, context);
        if table.is_null() {
            return sequences;
        }
        let state = (xkb.xkb_compose_state_new)(
            table,
            xkb_compose_state_flags::XKB_COMPOSE_STATE_NO_FLAGS,
        );
        if state.is_null() {
            (xkb.xkb_compose_table_unref)(table);
            return sequences;
        }
        // Keys that leave the compose state in `COMPOSING` introduce a
        // sequence: dead keys, and the compose key itself.
        let prefixes: Vec<&Candidate> = candidates
            .iter()
            .filter(|candidate| compose_eval(xkb, state, &[candidate.keysym]) == ComposeStep::Composing)
            .collect();

        for prefix in &prefixes {
            for candidate in candidates {
                let keysyms = [prefix.keysym, candidate.keysym];
                match compose_eval(xkb, state, &keysyms) {
                    ComposeStep::Composed(ch) => {
                        sequences.entry(ch).or_insert_with(|| {
                            vec![prefix.chord.clone(), candidate.chord.clone()]
                        });
                    }
                    ComposeStep::Composing => {
                        // Still composing: this pair needs a third key.
                        for third in candidates {
                            let keysyms = [prefix.keysym, candidate.keysym, third.keysym];
                            if let ComposeStep::Composed(ch) = compose_eval(xkb, state, &keysyms) {
                                sequences.entry(ch).or_insert_with(|| {
                                    vec![prefix.chord.clone(), candidate.chord.clone(), third.chord.clone()]
                                });
                            }
                        }
                    }
                    ComposeStep::Other => {}
                }
            }
        }

        (xkb.xkb_compose_state_unref)(state);
        (xkb.xkb_compose_table_unref)(table);
        sequences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typist_available() -> bool {
        xkbcommon_option().is_some()
    }

    #[test]
    fn compose_only_maps_complete_single_characters() {
        assert_eq!(composed_character("é\0".as_bytes(), 2), Some('é'));
        assert_eq!(composed_character("😀\0".as_bytes(), 4), Some('😀'));
        assert_eq!(composed_character(b"ab\0", 2), None);
        assert_eq!(composed_character(b"\xff\0", 1), None);
        assert_eq!(composed_character(b"\n\0", 1), None);
        assert_eq!(composed_character(&[0; 64], 63), None);
        assert_eq!(composed_character(&[0; 64], 64), None);
        assert_eq!(composed_character(&[0; 64], 1000), None);
        assert_eq!(composed_character(&[0; 64], -1), None);
    }

    #[test]
    fn custom_compose_outputs_are_rejected() {
        if !typist_available() || xkbcommon_compose_option().is_none() {
            return;
        }
        // Isolate XCOMPOSEFILE in a subprocess: changing it in the parallel
        // test runner would affect unrelated layout tests and the user locale.
        if std::env::var_os("XREMAP_TEST_CUSTOM_COMPOSE").is_some() {
            let mut typist = Typist::new();
            assert!(typist.chords(Some("de"), None, TextMethod::Layout, "中").is_err());
            assert!(typist.chords(Some("de"), None, TextMethod::Layout, "字").is_err());
            // Ω also has a direct key on de; inspect the sequence table to
            // verify the custom Compose file was actually loaded.
            assert_eq!(typist.keymap("de", "").unwrap().unwrap().sequences[&'Ω'].len(), 2);
            return;
        }
        let dir = std::env::temp_dir().join(format!("xremap-compose-{}-{:x}", std::process::id(), fastrand::u64(..)));
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join("XCompose");
        let contents = format!("<dead_acute> <e> : \"中文\"\n<dead_acute> <a> : \"{}\"\n<dead_acute> <o> : \"Ω\"\n", "字".repeat(80));
        std::fs::write(&file, contents).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "typist::tests::custom_compose_outputs_are_rejected", "--nocapture"])
            .env("XREMAP_TEST_CUSTOM_COMPOSE", "1")
            .env("XCOMPOSEFILE", &file)
            .env("LC_ALL", "en_US.UTF-8")
            .output().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }

    #[test]
    fn types_ascii_letters_with_shift() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        let chords = typist.chords(Some("us"), None, TextMethod::Layout, "aA").unwrap();
        assert_eq!(chords[0], CharChord { modifiers: vec![], key: Key::KEY_A });
        assert_eq!(
            chords[1],
            CharChord { modifiers: vec![Key::KEY_LEFTSHIFT], key: Key::KEY_A }
        );
    }

    #[test]
    fn types_digits_and_symbols() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        let chords = typist.chords(Some("us"), None, TextMethod::Layout, "1!").unwrap();
        assert_eq!(chords[0], CharChord { modifiers: vec![], key: Key::KEY_1 });
        assert_eq!(
            chords[1],
            CharChord { modifiers: vec![Key::KEY_LEFTSHIFT], key: Key::KEY_1 }
        );
    }

    #[test]
    fn newline_and_tab_are_dedicated_keys() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        let chords = typist.chords(Some("us"), None, TextMethod::Layout, "\n\t").unwrap();
        assert_eq!(chords[0].key, Key::KEY_ENTER);
        assert_eq!(chords[1].key, Key::KEY_TAB);
    }

    #[test]
    fn dead_keys_type_accents() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        // `de` has a dead_acute key: `´` then `e` gives `é`.
        let chords = typist.chords(Some("de"), None, TextMethod::Layout, "é").unwrap();
        assert_eq!(chords.len(), 2, "expected a dead-key sequence: {chords:?}");
        // The first chord must be a bare key, the second a bare key too.
        assert!(chords[0].modifiers.is_empty());

        // A three-part sequence: dead_circumflex + dead_acute + a = ấ.
        let chords = typist.chords(Some("de"), None, TextMethod::Layout, "ấ").unwrap();
        assert_eq!(chords.len(), 3, "expected a three-part sequence: {chords:?}");
    }

    #[test]
    fn dead_keys_respect_the_layout() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        // Plain `us` has no dead keys, so `é` is not reachable with real keys.
        assert!(typist.chords(Some("us"), None, TextMethod::Layout, "é").is_err());
        // `us(intl)` does.
        let chords = typist.chords(Some("us"), Some("intl"), TextMethod::Layout, "é");
        assert!(chords.is_ok(), "us(intl) should reach é: {chords:?}");
    }

    #[test]
    fn layout_specific_characters() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        // German umlaut: not reachable on us.
        assert!(typist.chords(Some("us"), None, TextMethod::Layout, "ü").is_err());
        let chords = typist.chords(Some("de"), None, TextMethod::Layout, "ü").unwrap();
        assert_eq!(chords.len(), 1);
    }

    #[test]
    fn auto_falls_back_to_ctrl_shift_u() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        // 中 = U+4E2D: Ctrl+Shift+U, 4, e, 2, d, Enter
        let chords = typist.chords(Some("us"), None, TextMethod::Auto, "中").unwrap();
        assert_eq!(chords.len(), 6);
        assert_eq!(
            chords[0],
            CharChord {
                modifiers: vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT],
                key: Key::KEY_U
            }
        );
        assert_eq!(chords[1].key, Key::KEY_4);
        assert_eq!(chords[2].key, Key::KEY_E);
        assert_eq!(chords[3].key, Key::KEY_2);
        assert_eq!(chords[4].key, Key::KEY_D);
        assert_eq!(chords[5], CharChord { modifiers: vec![], key: Key::KEY_ENTER });
    }

    #[test]
    fn auto_prefers_the_layout() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        // `ü` exists on `de`, so no Unicode sequence is involved.
        let chords = typist.chords(Some("de"), None, TextMethod::Auto, "ü").unwrap();
        assert_eq!(chords.len(), 1);
    }

    #[test]
    fn unicode_method_forces_the_sequence_for_ascii() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        let chords = typist.chords(Some("us"), None, TextMethod::Unicode, "a").unwrap();
        // 'a' = U+61: Ctrl+Shift+U, 6, 1, Enter
        assert_eq!(chords.len(), 4);
        assert_eq!(chords[1].key, Key::KEY_6);
        assert_eq!(chords[2].key, Key::KEY_1);
    }

    #[test]
    fn layout_method_rejects_unsupported_characters() {
        if !typist_available() {
            return;
        }
        let mut typist = Typist::new();
        let err = typist.chords(Some("us"), None, TextMethod::Layout, "a中").unwrap_err();
        assert!(err.contains("not available on layout"), "{err}");
    }
}
