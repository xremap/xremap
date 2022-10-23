use crate::event::KeyEvent;

// Input to ActionDispatcher. This should only contain things that are easily testable.
pub enum Action {
    // InputEvent (EventType::KEY) sent to evdev
    KeyEvent(KeyEvent),
}
