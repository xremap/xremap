use crate::config::application::Application;
use crate::config::key::Key;
use crate::config::key_action::KeyAction;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modmap {
    #[serde(default = "String::new")]
    pub name: String,
    pub remap: HashMap<Key, KeyAction>,
    pub application: Option<Application>,
}
