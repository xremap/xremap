use crate::event_handler::{MODIFIER_KEYS, PRESS, RELEASE};
use evdev::KeyCode;
use log::debug;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct ThrottleEmit {
    delay: Duration,
    // When modifier was last either pressed or released.
    last_mod: Instant,
    // When any ordinary key was last pressed.
    last_key_press: Instant,
    last_specific_key_press: HashMap<u16, Instant>,
}

/// Delay (if needed) between press and release of the same key. But not the other way around.
/// Delay (if needed) between press of ordinary key and press/release of modifier key.
/// Delay (if needed) between press/release of modifier key and press of ordinary key.
impl ThrottleEmit {
    pub fn new(delay: Duration) -> ThrottleEmit {
        ThrottleEmit {
            delay,
            last_mod: Instant::now(),
            last_key_press: Instant::now(),
            last_specific_key_press: HashMap::new(),
        }
    }

    pub fn sleep_if_needed(&mut self, key: KeyCode, value: i32) {
        if MODIFIER_KEYS.contains(&key) {
            if value == PRESS || value == RELEASE {
                self.last_mod = Instant::now();

                self.sleep(self.last_key_press.elapsed())
            }
        } else {
            if value == RELEASE {
                match self.last_specific_key_press.get(&key.0) {
                    Some(last_press) => self.sleep(last_press.elapsed()),
                    None => {
                        // nothing to do
                    }
                }
            } else if value == PRESS {
                self.last_key_press = Instant::now();
                self.last_specific_key_press.insert(key.0, Instant::now());

                self.sleep(self.last_mod.elapsed())
            }
        };
    }

    fn sleep(&self, elapsed: Duration) {
        if elapsed < self.delay {
            debug!("Delay: {:?}", self.delay - elapsed);
            sleep(self.delay - elapsed);
        }
    }
}
