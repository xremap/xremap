use super::device::Device;
use crate::config::application::deserialize_string_or_vec;
use crate::config::application::OnlyOrNot;
use crate::config::key::deserialize_key;
use crate::config::modmap_operator::ModmapOperator;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modmap {
    #[allow(dead_code)]
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<Key, ModmapOperator>,
    pub application: Option<OnlyOrNot>,
    pub window: Option<OnlyOrNot>,
    pub device: Option<Device>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub mode: Option<Vec<String>>,
}

#[derive(Deserialize, Eq, Hash, PartialEq)]
pub struct KeyWrapper(#[serde(deserialize_with = "deserialize_key")] pub Key);

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, ModmapOperator>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = HashMap::<KeyWrapper, ModmapOperator>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(KeyWrapper(k), v)| (k, v)).collect())
}
