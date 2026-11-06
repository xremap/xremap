use crate::action::Action;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::ipc_server::InjectScope;
use crate::tests::assert_injection;
use evdev::KeyCode as Key;
use indoc::indoc;

// Scope = the pipeline layer the key enters at; from there it flows to the
// app. Layers: experimental_map operators → modmap → keymap → app.
static CONFIG: &str = indoc! {"
experimental_map:
  - remap:
      x:
        double: ['1', '2']
        timeout: 10
modmap:
  - remap:
      f13: f14
keymap:
  - remap:
      f13: f15
      f14: right
"};

fn press_release(key: Key) -> Vec<Event> {
    vec![Event::key_press(key), Event::key_release(key)]
}

// raw → app: nothing reaches keymap or modmap (asserted at the client level).
#[test]
fn inject_keymap_scope_enters_at_keymap() {
    // f13 → keymap f13: f15. The trailing F13 release and zero delays are
    // exactly what a physical keypress produces through the same keymap.
    assert_injection(
        CONFIG,
        InjectScope::Keymap,
        press_release(Key::KEY_F13),
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_F15, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F15, KeyValue::Release)),
            Action::Delay(std::time::Duration::ZERO),
            Action::Delay(std::time::Duration::ZERO),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F13, KeyValue::Release)),
        ],
    );
}

// modmap → modmap → keymap → app: f13 →f14→ keymap f14: right.
#[test]
fn inject_modmap_scope_enters_at_modmap() {
    assert_injection(
        CONFIG,
        InjectScope::Modmap,
        press_release(Key::KEY_F13),
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(std::time::Duration::ZERO),
            Action::Delay(std::time::Duration::ZERO),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F14, KeyValue::Release)),
        ],
    );
}

// modmap scope skips experimental_map operators: x passes through untouched.
#[test]
fn inject_modmap_scope_skips_expmap() {
    assert_injection(
        CONFIG,
        InjectScope::Modmap,
        press_release(Key::KEY_X),
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
        ],
    );
}

// full scope goes through the expmap operators: x is swallowed by the
// double-tap operator (buffered until its timeout, no immediate output).
#[test]
fn inject_full_scope_enters_at_expmap() {
    assert_injection(CONFIG, InjectScope::Expmap, press_release(Key::KEY_X), vec![]);
}
