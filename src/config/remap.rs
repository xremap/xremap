use crate::config::action::Action;
use crate::config::key_press::KeyPress;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub timeout: Option<Duration>,
}
