use std::time::{Duration, Instant};
use xremap::{xremap_cli, KeyCode, KeyValue, Plugin, Result};

const KEY_TO_SQUASH: KeyCode = KeyCode::KEY_KPMINUS;
const SQUASH_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<()> {
    let handler = CustomPlugin {
        // In the past to start non-squashed
        squash_until: Instant::now() - Duration::from_hours(1),
    };

    xremap_cli(handler)
}

pub struct CustomPlugin {
    // Squash the key until this instant is reached.
    squash_until: Instant,
}

impl Plugin for CustomPlugin {
    fn on_key_event(&mut self, key: KeyCode, value: KeyValue) -> Vec<(KeyCode, KeyValue)> {
        if key == KEY_TO_SQUASH {
            if value == KeyValue::Press || value == KeyValue::Repeat {
                if Instant::now() < self.squash_until {
                    // Keep squashing
                    return vec![];
                } else {
                    // Start new period of squashing
                    self.squash_until = Instant::now() + SQUASH_TIME
                }
            }
        }

        vec![(key, value)]
    }
}
