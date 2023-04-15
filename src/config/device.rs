use serde::Deserialize;
use crate::config::application::deserialize_string_or_vec;

// TODO: Use trait to allow only either `only` or `not`
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub only: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub not: Option<Vec<String>>,
}
