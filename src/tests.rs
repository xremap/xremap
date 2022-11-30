use evdev::EventType;
use evdev::InputEvent;
use evdev::Key;
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::time::Duration;

use crate::client::{Client, WMClient};
use crate::{
    action::Action,
    config::{keymap::build_keymap_table, Config},
    event::{Event, KeyEvent, KeyValue, RelativeEvent},
    event_handler::EventHandler,
};

struct StaticClient {
    current_application: Option<String>,
}

impl Client for StaticClient {
    fn supported(&mut self) -> bool {
        true
    }

    fn current_application(&mut self) -> Option<String> {
        self.current_application.clone()
    }
}

#[test]
fn test_basic_modmap() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              a: b
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

/* Table to see which scancodes/custom key events correspond to which relative events
    Original RELATIVE event | scancode | Custom keyname if                              | Info
                            |          | positive value (+)     | negative value (-)    |
    REL_X                   |    0     | BTN_XRIGHTCURSOR       | BTN_XLEFTCURSOR       | Cursor right and left
    REL_Y                   |    1     | BTN_XDOWNCURSOR        | BTN_XUPCURSOR         | Cursor down and up
    REL_Z                   |    2     | BTN_XREL_Z_AXIS_1      | BTN_XREL_Z_AXIS_2     | Cursor... forward and backwards?
    REL_RX                  |    3     | BTN_XREL_RX_AXIS_1     | BTN_XREL_RX_AXIS_2    | Horizontally rotative cursor movement?
    REL_RY                  |    4     | BTN_XREL_RY_AXIS_1     | BTN_XREL_RY_AXIS_2    | Vertical rotative cursor movement?
    REL_RZ                  |    5     | BTN_XREL_RZ_AXIS_1     | BTN_XREL_RZ_AXIS_2    | "Whatever the third dimensional axis is called" rotative cursor movement?
    REL_HWHEEL              |    6     | BTN_XRIGHTSCROLL       | BTN_XLEFTSCROLL       | Rightscroll and leftscroll
    REL_DIAL                |    7     | BTN_XREL_DIAL_1        | BTN_XREL_DIAL_2       | ???
    REL_WHEEL               |    8     | BTN_XUPSCROLL          | BTN_XDOWNSCROLL       | Upscroll and downscroll
    REL_MISC                |    9     | BTN_XREL_MISC_1        | BTN_XREL_MISC_2       | Something?
    REL_RESERVED            |    10    | BTN_XREL_RESERVED_1    | BTN_XREL_RESERVED_2   | Something?
    REL_WHEEL_HI_RES        |    11    | BTN_XHIRES_UPSCROLL    | BTN_XHIRES_DOWNSCROLL | High resolution downscroll and upscroll, sent just after their non-high resolution version
    REL_HWHEEL_HI_RES       |    12    | BTN_XHIRES_RIGHTSCROLL | BTN_XHIRES_LEFTSCROLL | High resolution rightcroll and leftscroll, sent just after their non-high resolution version
*/

const _POSITIVE: i32 = 1;
const _NEGATIVE: i32 = -1;

const _REL_X: u16 = 0;
const _REL_Y: u16 = 1;
const _REL_Z: u16 = 2;
const _REL_RX: u16 = 3;
const _REL_RY: u16 = 4;
const _REL_RZ: u16 = 5;
const _REL_HWHEEL: u16 = 6;
const _REL_DIAL: u16 = 7;
const _REL_WHEEL: u16 = 8;
const _REL_MISC: u16 = 9;
const _REL_RESERVED: u16 = 10;
const _REL_WHEEL_HI_RES: u16 = 11;
const _REL_HWHEEL_HI_RES: u16 = 12;

#[test]
fn test_relative_events() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              BTN_XRIGHTCURSOR: b
        "},
        vec![Event::RelativeEvent(RelativeEvent::new_with(_REL_X, _POSITIVE))],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

#[test]
#[ignore]
// The OS interprets a REL_X¹ event combined with a REL_Y event² differently if they are separated by synchronization event.
// This test and test_cursor_behavior_2 are meant to be run to demonstrate that fact.

//¹Mouse movement along the X (horizontal) axis.
//²Mouse movement along the Y (vertical) axis.

// The only difference between test_cursor_behavior_1 and test_cursor_behavior_2 is that
// test_cursor_behavior_1 adds a synchronization event between REL_X and REL_Y events that would not normally be there.
// In other words, test_cursor_behavior_2 represents what would occur without Xremap intervention.

// Here's how to proceed :
// 1 - Move your mouse cursor to the bottom left of your screen.
// 2 - either run this test with sudo privileges or while your environnment is properly set up (https://github.com/k0kubun/xremap#running-xremap-without-sudo),
//     so that your keyboard and/or mouse may be captured.

// 3 - Press any button (don't move the mouse).
// 4 - Note where the cursor ended up.

// 5 - Repeat steps 1 through 4 for test_cursor_behavior_2.
// 6 - Notice that the mouse cursor often ends up in a different position than when running test_cursor_behavior_1.

//
// Notes :
// - Because emitting an event automatcially adds a synchronization event afterwards (see https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/uinput.rs#L167),
//   Mouse movement events should be batched together when emitted,
//   to avoid separating them with a synchronization event.
//
// - Because a mouse will only ever send a maximum of one REL_X and one REL_Y (and maybe one REL_Z for 3D mice?) at once,
//   the only point where a synchronization event can be added where it shouldn't by Xremap is between those events,
//   meaning this bug is exclusive to diagonal mouse movement.
//
// - The call to std::thread::sleep for five milliseconds is meant to emulate
//   the interval between events from a mouse with a frequency of ~200 Hz.
//   A lower time interval between events (which would correspond to a mouse with a higher frequency)
//   would cause the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to become less noticeable.
//   Conversely, a higher time interval would make the difference more noticeable.
//
fn test_cursor_behavior_1() {
    use crate::device::InputDevice;
    use crate::device::{get_input_devices, output_device};
    //Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {}", e),
    };
    let mut output_device = match output_device(input_devices.values().next().map(InputDevice::bus_type)) {
        Ok(output_device) => output_device,
        Err(e) => panic!("Failed to prepare an output device: {}", e),
    };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    //Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE, _REL_X, _POSITIVE),
                //
                //This line is the only difference between test_cursor_behavior_1 and test_cursor_behavior_2.
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                //
                InputEvent::new_now(EventType::RELATIVE, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        //Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        //The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
#[ignore]
// The OS interprets a REL_X¹ event combined with a REL_Y event² differently if they are separated by synchronization event.
// This test and test_cursor_behavior_1 are meant to be run to demonstrate that fact.
// Please refer to the comment above test_cursor_behavior_1 for information on how to run these tests.
fn test_cursor_behavior_2() {
    use crate::device::InputDevice;
    use crate::device::{get_input_devices, output_device};
    //Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {}", e),
    };
    let mut output_device = match output_device(input_devices.values().next().map(InputDevice::bus_type)) {
        Ok(output_device) => output_device,
        Err(e) => panic!("Failed to prepare an output device: {}", e),
    };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    //Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE, _REL_X, _POSITIVE),
                InputEvent::new_now(EventType::RELATIVE, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        //Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        //The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn test_interleave_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
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
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_true_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_exact_match_false_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: false
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
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
        vec![Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions_with_current_application(
        config,
        Some(String::from("firefox")),
        vec![Event::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press))],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_merge_remaps() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  k: C-w
    "};

    assert_actions(
        config,
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_merge_remaps_with_override() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  h: C-b
                  c: C-q
    "};

    assert_actions(
        config,
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

fn assert_actions(config_yaml: &str, events: Vec<Event>, actions: Vec<Action>) {
    assert_actions_with_current_application(config_yaml, None, events, actions);
}

fn assert_actions_with_current_application(
    config_yaml: &str,
    current_application: Option<String>,
    events: Vec<Event>,
    actions: Vec<Action>,
) {
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut config: Config = serde_yaml::from_str(config_yaml).unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    let mut event_handler = EventHandler::new(
        timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application })),
    );
    let mut actual: Vec<Action> = vec![];
    for event in &events {
        actual.append(&mut event_handler.on_event(event, &config).unwrap());
    }
    assert_eq!(format!("{:?}", actions), format!("{:?}", actual));
}
