use crate::config::application::deserialize_string_or_vec;
use crate::config::application::OnlyOrNot;
use crate::config::key::deserialize_key;
use crate::config::modmap_action::ModmapAction;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use super::device::Device;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modmap {
    #[allow(dead_code)]
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<Key, ModmapAction>,
    pub application: Option<OnlyOrNot>,
    pub window: Option<OnlyOrNot>,
    pub device: Option<Device>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub mode: Option<Vec<String>>,
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, ModmapAction>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize, Eq, Hash, PartialEq)]
    struct KeyWrapper(#[serde(deserialize_with = "deserialize_key")] Key);

    let v = HashMap::<KeyWrapper, ModmapAction>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(KeyWrapper(k), v)| (k, v)).collect())
}
