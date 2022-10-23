use crate::event::KeyEvent;

pub enum Action {
    // InputEvent::Key sent to evdev
    KeyEvent(KeyEvent),
}
