use crate::config::application::Application;
use crate::config::key_press::KeyPress;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hotkey {
    #[serde(default = "String::new")]
    pub name: String,
    pub keys: Vec<KeyPress>,
    pub command: Vec<String>,
    pub application: Option<Application>,
}
