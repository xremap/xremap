use crate::config::modmap::KeyWrapper;
use crate::config::ConfigFiletype;
use evdev::KeyCode as Key;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::error;

static mut ALIASES: Option<HashMap<String, Key>> = None;

#[derive(Deserialize)]
struct AliasConfig {
    // Using IndexMap makes sure two aliases with same LHS always
    // use the last definition. It's just easier when it's deterministic.
    // It can easily happen for uppercase/lowercase, which is considered the same.
    #[serde(default)]
    pub key_aliases: IndexMap<String, KeyWrapper>,
}

/// The aliases must be set before the rest of the config is parsed.
/// And it's only possible by parsing the config twice.
pub fn pre_load_key_aliases(content: &str, file_type: ConfigFiletype) -> Result<(), Box<dyn error::Error>> {
    // Dummy to allow calls to `parse_key` when parsing the aliases.
    unsafe { ALIASES = Some(HashMap::new()) };

    let config: AliasConfig = match file_type {
        ConfigFiletype::Yaml => serde_yaml::from_str(content)?,
        ConfigFiletype::Toml => toml::from_str(content)?,
    };

    let aliases = config
        .key_aliases
        .into_iter()
        .map(|(alias, KeyWrapper(key))| (alias.to_uppercase(), key))
        .collect();

    unsafe { ALIASES = Some(aliases) };

    Ok(())
}

pub fn lookup_key_alias(key_name: &str) -> Option<Key> {
    unsafe {
        #[allow(static_mut_refs)]
        match &ALIASES {
            Some(map) => map.get(&key_name.to_uppercase()).cloned(),
            None => {
                // `pre_load_key_aliases` must have been called first.
                unreachable!();
            }
        }
    }
}
