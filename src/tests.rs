use crate::action::Action;
use crate::client::WindowInfo;
use crate::client::{Client, WMClient};
use crate::config::keymap::build_keymap_table;
use crate::config::{validate_config_file, Config};
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue, RelativeEvent};
use crate::event_handler::EventHandler;
use evdev::{KeyCode as Key, RelativeAxisCode};
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

/// There are a lot of features and some interact.
/// To order test cases they are placed according to which features they are testing.
/// Test cases are placed in the file of the most specific feature they test.
/// With the following definition of specific (ordered by most specific first):
///
///     Any key
///     Virtual modifiers
///     Disguised events input (i.e. transformation of relative event to pseudo keys)
///     Multipurpose keys (tap-preferred)
///     Multipurpose keys (hold-preferred)
///     PressRelease keys
///     Modmap key-to-key
///     Nested remap in keymap
///     Keymap
///
/// In other words: A feature is responsible for testing
/// all interactions with less specific features.

struct StaticClient {
    current_application: Option<String>,
}

impl Client for StaticClient {
    fn supported(&mut self) -> bool {
        true
    }
    fn current_window(&mut self) -> Option<String> {
        None
    }

    fn current_application(&mut self) -> Option<String> {
        self.current_application.clone()
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(vec![])
    }

    fn close_windows_by_app_class(&mut self, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn get_input_device_info() -> Rc<InputDeviceInfo> {
    Rc::new(InputDeviceInfo {
        name: "Some Device".into(),
        path: PathBuf::from("/dev/input/event0"),
        vendor: 0x1234,
        product: 0x5678,
    })
}

#[test]
fn test_mouse_movement_event_accumulation() {
    assert_actions(
        "",
        vec![
            Event::relative(RelativeAxisCode::REL_X.0, 1),
            Event::relative(RelativeAxisCode::REL_Y.0, 1),
        ],
        vec![Action::MouseMovementEventCollection(vec![
            RelativeEvent::new_with(RelativeAxisCode::REL_X.0, 1),
            RelativeEvent::new_with(RelativeAxisCode::REL_Y.0, 1),
        ])],
    )
}

#[test]
fn test_interleave_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![Event::key_press(Key::KEY_LEFTALT), Event::key_press(Key::KEY_F)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_true() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              M-f: C-right
        "},
        vec![
            Event::key_press(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_F),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_exact_match_false() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: false
            remap:
              M-f: C-right
        "},
        vec![
            Event::key_press(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_F),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_default() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![
            Event::key_press(Key::KEY_LEFTALT),
            Event::key_press(Key::KEY_LEFTSHIFT),
            Event::key_press(Key::KEY_F),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_keymaps_are_merged() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a: b
          - remap:
              c: d
        "},
        vec![
            Event::key_press(Key::KEY_A),
            Event::key_release(Key::KEY_A),
            Event::key_press(Key::KEY_C),
            Event::key_release(Key::KEY_C),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_keymap_merge_gives_precedence_to_first() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a: b
          - remap:
              a: c
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_keymap_emit_is_not_used_in_subsequent_remaps() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                a: b
            - remap:
                b: c
        "},
        vec![Event::key_press(Key::KEY_A), Event::key_release(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_application_override() {
    let config = indoc! {"
        keymap:

          - name: firefox
            application:
              only: [firefox]
            remap:
              a: C-c

          - name: generic
            remap:
              a: C-b
    "};

    assert_actions(
        config,
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions_with_current_application(
        config,
        Some(String::from("firefox")),
        vec![Event::key_press(Key::KEY_A)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_device_override() {
    let config = indoc! {"
        keymap:

          - name: event1
            device:
              only: [event1]
            remap:
              a: C-c

          - name: event0
            remap:
              a: C-b
    "};

    assert_actions(
        config,
        vec![Event::KeyEvent(
            Rc::new(InputDeviceInfo {
                name: "Some Device".into(),
                path: PathBuf::from("/dev/input/event0"),
                vendor: 0x1234,
                product: 0x5678,
            }),
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![Event::KeyEvent(
            Rc::new(InputDeviceInfo {
                name: "Other Device".into(),
                path: PathBuf::from("/dev/input/event1"),
                vendor: 0x1234,
                product: 0x5678,
            }),
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_no_keymap_action() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_release(Key::KEY_F12)],
        vec![
            //This is just release, so the key is not emitted.
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
    );

    //Same test with the null keyword
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: null
        "},
        vec![Event::key_press(Key::KEY_F12), Event::key_release(Key::KEY_F12)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release))],
    )
}

#[test]
fn test_keymap_with_modifier_alone_is_not_supported() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              C_L: end
        "},
        vec![Event::key_press(Key::KEY_LEFTCTRL)],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press))],
    )
}

#[test]
fn test_keymap_modifier_spuriously_pressed() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                c_l-s: k
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            // Pressing a second time is an input error, but it can easily happen when doing remapping.
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_S),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_keymap_modifiers_are_released_in_order_of_pressed() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                c_l-w_l-s: k
        "},
        vec![
            Event::key_press(Key::KEY_LEFTCTRL),
            Event::key_press(Key::KEY_LEFTMETA),
            Event::key_press(Key::KEY_S),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
        ],
    )
}

#[test]
fn test_keymap_press_release_repeat_only_actions() {
    assert_actions(
        indoc! {"
        keymap:
            - remap:
                capslock:
                    - { press: A}
                    - { release: B}
                    - { repeat: C}
        "},
        vec![Event::key_press(Key::KEY_CAPSLOCK)],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Repeat)),
        ],
    )
}

#[test]
fn test_keymap_action_error() {
    let errmsg = serde_yaml::from_str::<Config>(indoc! {"
    keymap:
        - remap:
            a: { not_a_keymap_action: foo }
    "})
    .unwrap_err()
    .to_string();

    assert_eq!(
        &errmsg,
        "keymap[0].remap: data did not match any variant of untagged enum Actions at line 3 column 9"
    );
}

pub fn assert_events(actual: impl AsRef<Vec<Event>>, expected: impl AsRef<Vec<Event>>) {
    let actual = actual.as_ref();
    let expected = expected.as_ref();

    assert_eq!(format!("{actual:?}"), format!("{:?}", expected));
}

pub fn assert_actions(config_yaml: &str, events: Vec<Event>, actions: Vec<Action>) {
    EventHandlerForTest::new_with_current_application(config_yaml, None).assert(events, actions);
}

pub fn assert_actions_with_current_application(
    config_yaml: &str,
    current_application: Option<String>,
    events: Vec<Event>,
    actions: Vec<Action>,
) {
    EventHandlerForTest::new_with_current_application(config_yaml, current_application).assert(events, actions);
}

pub struct EventHandlerForTest {
    event_handler: EventHandler,
    config: Config,
    wmclient: WMClient,
}

impl EventHandlerForTest {
    pub fn new(config_yaml: &str) -> Self {
        Self::new_with_current_application(config_yaml, None)
    }

    pub fn new_with_current_application(config_yaml: &str, current_application: Option<String>) -> Self {
        let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
        let mut config: Config = serde_yaml::from_str(config_yaml).unwrap();
        config.keymap_table = build_keymap_table(&config.keymap);
        validate_config_file(&config).unwrap();
        let event_handler = EventHandler::new(timer, &config.default_mode, Duration::from_micros(0));

        Self {
            event_handler,
            config,
            wmclient: WMClient::new("static", Box::new(StaticClient { current_application }), false),
        }
    }

    pub fn assert(&mut self, events: Vec<Event>, actions: Vec<Action>) {
        assert_eq!(
            format!("{actions:?}"),
            format!(
                "{:?}",
                self.event_handler
                    .on_events(&events, &self.config, &mut self.wmclient)
                    .unwrap()
            )
        );
    }
}
