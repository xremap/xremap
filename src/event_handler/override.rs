use std::error::Error;

use super::EventHandler;
use crate::action::Action;
use crate::event::KeyValue;

impl EventHandler {
    pub fn timeout_override(&mut self) -> Result<Vec<Action>, Box<dyn Error>> {
        let mut actions = Vec::new();
        if let Some(keys) = &self.override_timeout_key.take() {
            for key in keys {
                actions.push(self.send_key(key, KeyValue::Press));
                actions.push(self.send_key(key, KeyValue::Release));
            }
        }
        self.remove_override()?;
        Ok(actions)
    }

    pub fn remove_override(&mut self) -> Result<(), Box<dyn Error>> {
        self.override_timer.unset()?;
        self.override_remaps.clear();
        self.override_timeout_key = None;
        Ok(())
    }
}
