use super::device::DeviceMatcher;
use crate::config::application::ApplicationMatch;
use crate::config::deserializers::{deserialize_key, deserialize_string_or_vec};
use crate::config::modmap_operator::{ModmapOperator, ParseModmapOperator};
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Modmap {
    pub remap: HashMap<Key, ModmapOperator>,
    pub application: Option<ApplicationMatch>,
    pub window: Option<ApplicationMatch>,
    pub device: Option<DeviceMatcher>,
    pub mode: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParseModmap {
    #[allow(dead_code)]
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<Key, ParseModmapOperator>,
    pub application: Option<ApplicationMatch>,
    pub window: Option<ApplicationMatch>,
    pub device: Option<DeviceMatcher>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub mode: Option<Vec<String>>,
}

impl ParseModmap {
    pub fn finish(modmaps: Vec<ParseModmap>) -> Vec<Modmap> {
        modmaps
            .into_iter()
            .map(|modmap| {
                let remap = modmap
                    .remap
                    .into_iter()
                    .map(|(key, operator)| (key, operator.finish()))
                    .collect();

                Modmap {
                    remap,
                    application: modmap.application,
                    window: modmap.window,
                    device: modmap.device,
                    mode: modmap.mode,
                }
            })
            .collect()
    }
}

#[derive(Deserialize, Eq, Hash, PartialEq)]
pub struct KeyWrapper(#[serde(deserialize_with = "deserialize_key")] pub Key);

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, ParseModmapOperator>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = HashMap::<KeyWrapper, ParseModmapOperator>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(KeyWrapper(k), v)| (k, v)).collect())
}
