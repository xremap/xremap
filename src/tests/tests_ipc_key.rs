use crate::action::Action;
use crate::event::Event;
use crate::tests::assert_actions;
use evdev::KeyCode as Key;
use indoc::indoc;

// `remap` scope goes through the normal pipeline, from where the key flows on
// to the app. Layers: experimental_map operators → modmap → keymap → app.
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

// remap → modmap → keymap → app: f13 →f14→ keymap f14: right. Entering at the
// top of the pipeline is exactly what a physical keypress produces.
#[test]
fn remap_scope_runs_modmap_and_keymap() {
    assert_actions(
        CONFIG,
        press_release(Key::KEY_F13),
        vec![
            Action::KeyEvent(crate::event::KeyEvent::new(Key::KEY_RIGHT, crate::KeyValue::Press)),
            Action::KeyEvent(crate::event::KeyEvent::new(Key::KEY_RIGHT, crate::KeyValue::Release)),
            Action::Delay(std::time::Duration::ZERO),
            Action::Delay(std::time::Duration::ZERO),
            Action::KeyEvent(crate::event::KeyEvent::new(Key::KEY_F14, crate::KeyValue::Release)),
        ],
    );
}

// remap goes through the experimental_map operators: x is swallowed by the
// double-tap operator (buffered until its timeout, no immediate output).
#[test]
fn remap_scope_runs_experimental_map_operators() {
    assert_actions(CONFIG, press_release(Key::KEY_X), vec![]);
}
