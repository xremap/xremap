use crate::config::deserializers::{VecOrSingle, VectorOrSingleOrNull};
use crate::config::key_combo::{EmitMode, KeyCombo, ParseCombo};
use crate::config::keymap_action::{KeymapAction, ParseKeymapAction};
use crate::config::simple_action::ParseSimpleAction;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: IndexMap<KeyCombo, Vec<KeymapAction>>,
    pub timeout: Option<Duration>,
    // Option is retained here, because it shows the user wants the default. The value can
    // only be determined at remap-time, because it depends on mod_trigger and any-key behavior.
    pub timeout_actions: Option<Vec<KeymapAction>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParseRemap {
    pub remap: IndexMap<ParseCombo, VectorOrSingleOrNull<ParseKeymapAction>>,
    pub timeout_millis: Option<u64>,
    #[serde(default, alias = "timeout_key", deserialize_with = "deserialize_timeout_key")]
    pub timeout: Option<Vec<ParseSimpleAction>>,
}

pub fn deserialize_timeout_key<'de, D>(deserializer: D) -> Result<Option<Vec<ParseSimpleAction>>, D::Error>
where
    D: Deserializer<'de>,
{
    // This disallows null, even though Option<T> would regard
    // missing fields and null the same.
    Ok(Some(VecOrSingle::<ParseSimpleAction>::deserialize(deserializer)?.into_vec()))
}

impl ParseRemap {
    pub fn finish(self) -> Remap {
        Remap {
            remap: self
                .remap
                .into_iter()
                .map(|(combo, actions)| (combo.as_is(), ParseKeymapAction::finish(actions.into_vec(), EmitMode::Combo)))
                .collect(),
            timeout: self.timeout_millis.map(Duration::from_millis),
            timeout_actions: self
                .timeout
                .map(|actions| ParseSimpleAction::finish(actions, EmitMode::Key)),
        }
    }
}
