use crate::event::{Event, KeyEvent, KeyValue};
use evdev::KeyCode as Key;

/// Customize remapping
///
/// # Example
///
/// ```
/// use xremap::{KeyCode, KeyValue, Plugin, Result, xremap_cli};
///
/// fn main() -> Result<()> {
///     # return Ok(()); // So it doesn't run in doctests.
///     xremap_cli(CustomPlugin {})
/// }
///
/// pub struct CustomPlugin {}
///
/// impl Plugin for CustomPlugin {
///     /// Remap each key event to itself.
///     fn on_key_event(&mut self, key: KeyCode, value: KeyValue)
///         -> Vec<(KeyCode, KeyValue)>
///     {
///         vec![(key, value)]
///     }
/// }
///
/// ```
pub trait Plugin {
    /// Whether the trait is considered implemented.
    ///
    /// Set to false to have the plugin completely remoted at compile-time.
    const IMPLEMENTED: bool = true;
    /// Remap key events.
    ///
    /// # Example
    ///
    /// ```
    /// # use xremap::{KeyCode, KeyValue};
    /// # struct CustomPlugin {}
    /// # impl CustomPlugin {
    /// fn on_key_event(&mut self, key: KeyCode, value: KeyValue)
    ///     -> Vec<(KeyCode, KeyValue)>
    /// {
    ///     if key == KeyCode::KEY_KPMINUS {
    ///         // Disable the key by dropping all events.
    ///         return vec![];
    ///     }
    ///     // Default
    ///     vec![(key, value)]
    /// }
    /// # }
    /// ```
    fn on_key_event(&mut self, key: Key, value: KeyValue) -> Vec<(Key, KeyValue)>;
}

pub struct NoopPlugin {}

impl Plugin for NoopPlugin {
    // Maybe unneeded way to avoid assembly code, when not implemented.
    const IMPLEMENTED: bool = false;
    fn on_key_event(&mut self, _: Key, _: KeyValue) -> Vec<(Key, KeyValue)> {
        unreachable!()
    }
}

pub fn apply_plugin(plugin: &mut impl Plugin, events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .flat_map(|event| match event {
            Event::KeyEvent(device, key_event) => plugin
                .on_key_event(key_event.key, key_event.value)
                .into_iter()
                .map(|(key, value)| Event::KeyEvent(device.clone(), KeyEvent::new(key, value)))
                .collect(),
            _ => {
                vec![event]
            }
        })
        .collect()
}
