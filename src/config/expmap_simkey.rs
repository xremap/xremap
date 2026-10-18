use crate::config::deserialize_keys;
use crate::config::deserializers::deserialize_duration;
use crate::config::expmap_actions::{ExpmapActions, ParseExpmapActions};
use evdev::KeyCode as Key;
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SimkeyDefinition {
    pub keys: Vec<Key>,
    pub actions: ExpmapActions,
    pub timeout: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Simkey {
    #[serde(deserialize_with = "deserialize_keys")]
    pub keys: Vec<Key>,
    #[serde(default = "default_symkey_timeout", deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
    pub actions: ParseExpmapActions,
}

fn default_symkey_timeout() -> Duration {
    Duration::from_millis(30)
}

impl Simkey {
    pub fn finish(self, bypass: bool) -> SimkeyDefinition {
        SimkeyDefinition {
            keys: self.keys,
            actions: self.actions.finish(bypass),
            timeout: self.timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tests::assert_invalid_config;
    use indoc::indoc;

    #[test]
    fn test_chord_with_one_key_fails() {
        assert_invalid_config(
            indoc! {"
                    experimental_map:
                      - chords:
                          - keys: [f11]
                            actions: A
                    "
            },
            "There must be at least two keys for a chord.",
        )
    }
}
