use std::time::Duration;

use evdev::InputEvent;

use crate::event::{KeyEvent, RelativeEvent};

// Input to ActionDispatcher. This should only contain things that are easily testable.
#[derive(Debug)]
pub enum Action {
    // InputEvent (EventType::KEY) sent to evdev
    KeyEvent(KeyEvent),
    // InputEvent (EventType::RELATIVE, NOT mouse movement events) sent to evdev
    RelativeEvent(RelativeEvent),
    // InputEvent (EventType::RELATIVE, ONLY mouse movement events) sent to evdev
    MouseMovementEvent(RelativeEvent),
    // InputEvent of any event types. It's discouraged to use this for testing because
    // we don't have full control over timeval and it's not pattern-matching friendly.
    InputEvent(InputEvent),
    // Run a command
    Command(Vec<String>),
    // keypress_delay_ms
    Delay(Duration),
}
