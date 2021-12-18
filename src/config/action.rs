use crate::config::keypress::KeyPress;

use serde::Deserialize;
use std::fmt::{Debug, Formatter};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Action {
    KeyPress(KeyPress),
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::KeyPress(key_press) => key_press.fmt(f),
        }
    }
}
