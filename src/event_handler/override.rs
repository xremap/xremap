use std::error::Error;

use super::EventHandler;

impl EventHandler {
    pub(super) fn timeout_override(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(keys) = &self.override_timeout_key.take() {
            for key in keys {
                self.send_key(&key, super::PRESS);
                self.send_key(&key, super::RELEASE);
            }
        }
        self.remove_override()
    }

    pub(super) fn remove_override(&mut self) -> Result<(), Box<dyn Error>> {
        self.override_timer.unset()?;
        self.override_remaps.clear();
        self.override_timeout_key = None;
        Ok(())
    }
}
