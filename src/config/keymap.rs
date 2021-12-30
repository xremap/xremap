use crate::config::action::{Action, Actions};
use crate::config::application::Application;
use crate::config::key_press::KeyPress;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub application: Option<Application>,
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<Action>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = HashMap::<KeyPress, Actions>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(k, v)| (k, v.to_vec())).collect())
}
