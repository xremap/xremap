use crate::config::application::OnlyOrNot;
use crate::config::modmap::KeyWrapper;
use crate::config::{expmap_operator::ExpmapOperator, expmap_simkey::Simkey};
use evdev::KeyCode as Key;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expmap {
    #[allow(dead_code)]
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub chords: Vec<Simkey>,
    #[serde(default, deserialize_with = "deserialize_experimental_remap")]
    pub remap: IndexMap<Key, ExpmapOperator>,
    pub application: Option<OnlyOrNot>,
    pub window: Option<OnlyOrNot>,
}

pub fn deserialize_experimental_remap<'de, D>(deserializer: D) -> Result<IndexMap<Key, ExpmapOperator>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = IndexMap::<KeyWrapper, ExpmapOperator>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(KeyWrapper(k), v)| (k, v)).collect())
}
