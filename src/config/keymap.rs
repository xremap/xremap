use super::device::DeviceMatcher;
use super::key_combo::Modifier;
use crate::config::application::ApplicationMatch;
use crate::config::deserializers::{deserialize_string_or_vec, VectorOrSingleOrNull};
use crate::config::key_combo::KeyCombo;
use crate::config::keymap_action::{KeymapAction, ParseKeymapAction};
use evdev::KeyCode as Key;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

// Config interface
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[allow(dead_code)]
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: IndexMap<KeyCombo, Vec<ParseKeymapAction>>,
    pub application: Option<ApplicationMatch>,
    pub window: Option<ApplicationMatch>,
    pub device: Option<DeviceMatcher>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub mode: Option<Vec<String>>,
    #[serde(default)]
    pub exact_match: bool,
}

impl Keymap {
    // Convert an array of keymaps to a single hashmap whose key is a triggering key.
    //
    // For each key, Vec<KeymapEntry> is scanned once, matching the exact modifiers,
    // and then it's scanned again, allowing extra modifiers.
    //
    // First matching KeymapEntry wins at each iteration.
    pub fn finish(keymaps: Vec<Keymap>) -> HashMap<Key, Vec<KeymapEntry>> {
        let mut table: HashMap<Key, Vec<KeymapEntry>> = HashMap::new();
        for keymap in keymaps {
            for (key_combo, actions) in keymap.remap {
                let entry = KeymapEntry {
                    actions: ParseKeymapAction::finish(actions),
                    modifiers: key_combo.modifiers,
                    application: keymap.application.clone(),
                    title: keymap.window.clone(),
                    device: keymap.device.clone(),
                    mode: keymap.mode.clone(),
                    exact_match: keymap.exact_match,
                };
                match table.get_mut(&key_combo.key) {
                    Some(entries) => {
                        entries.push(entry);
                    }
                    None => {
                        table.insert(key_combo.key, vec![entry]);
                    }
                };
            }
        }
        table
    }
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<IndexMap<KeyCombo, Vec<ParseKeymapAction>>, D::Error>
where
    D: Deserializer<'de>,
{
    // IndexMap preserves the order from the config file, which is why it's used instead of HashMap.
    let remap = IndexMap::<KeyCombo, VectorOrSingleOrNull<ParseKeymapAction>>::deserialize(deserializer)?;
    Ok(remap
        .into_iter()
        .map(|(key_combo, actions)| (key_combo, actions.into_vec()))
        .collect())
}

// Internals for efficient keymap lookup
#[derive(Clone, Debug)]
pub struct KeymapEntry {
    pub actions: Vec<KeymapAction>,
    pub modifiers: Vec<Modifier>,
    pub application: Option<ApplicationMatch>,
    pub title: Option<ApplicationMatch>,
    pub device: Option<DeviceMatcher>,
    pub mode: Option<Vec<String>>,
    pub exact_match: bool,
}

// This is executed on runtime unlike build_keymap_table, but hopefully not called so often.
pub fn build_override_table(
    remap: &IndexMap<KeyCombo, Vec<KeymapAction>>,
    exact_match: bool,
) -> HashMap<Key, Vec<KeymapEntry>> {
    let mut table: HashMap<Key, Vec<KeymapEntry>> = HashMap::new();
    for (key_combo, actions) in remap.iter() {
        let mut entries: Vec<KeymapEntry> = match table.get(&key_combo.key) {
            Some(entries) => entries.clone(),
            None => vec![],
        };
        entries.push(KeymapEntry {
            actions: actions.to_vec(),
            modifiers: key_combo.modifiers.clone(),
            application: None,
            title: None,
            device: None,
            mode: None,
            exact_match,
        });
        table.insert(key_combo.key, entries);
    }
    table
}
