use crate::config::modmap::KeyWrapper;
use crate::config::{expmap_operator::ExpmapOperator, expmap_simkey::Simkey};
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expmap {
    #[allow(dead_code)]
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub chords: Vec<Simkey>,
    #[serde(default, deserialize_with = "deserialize_experimental_remap")]
    pub remap: HashMap<Key, ExpmapOperator>,
}

pub fn deserialize_experimental_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, ExpmapOperator>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = HashMap::<KeyWrapper, ExpmapOperator>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(KeyWrapper(k), v)| (k, v)).collect())
}
