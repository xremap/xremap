use evdev::EventType;
use evdev::InputEvent;
use evdev::KeyCode as Key;
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::path::Path;
use std::time::Duration;

use crate::client::{Client, WMClient};
use crate::device::InputDeviceInfo;
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
    fn current_window(&mut self) -> Option<String> {
        None
    }

    fn current_application(&mut self) -> Option<String> {
        self.current_application.clone()
    }
}

fn get_input_device_info<'a>() -> InputDeviceInfo<'a> {
    InputDeviceInfo {
        name: "Some Device",
        path: &Path::new("/dev/input/event0"),
        vendor: 0x1234,
        product: 0x5678,
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Release)),
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
    REL_X                   |    0     | XRIGHTCURSOR       | XLEFTCURSOR       | Cursor right and left
    REL_Y                   |    1     | XDOWNCURSOR        | XUPCURSOR         | Cursor down and up
    REL_Z                   |    2     | XREL_Z_AXIS_1      | XREL_Z_AXIS_2     | Cursor... forward and backwards?
    REL_RX                  |    3     | XREL_RX_AXIS_1     | XREL_RX_AXIS_2    | Horizontally rotative cursor movement?
    REL_RY                  |    4     | XREL_RY_AXIS_1     | XREL_RY_AXIS_2    | Vertical rotative cursor movement?
    REL_RZ                  |    5     | XREL_RZ_AXIS_1     | XREL_RZ_AXIS_2    | "Whatever the third dimensional axis is called" rotative cursor movement?
    REL_HWHEEL              |    6     | XRIGHTSCROLL       | XLEFTSCROLL       | Rightscroll and leftscroll
    REL_DIAL                |    7     | XREL_DIAL_1        | XREL_DIAL_2       | ???
    REL_WHEEL               |    8     | XUPSCROLL          | XDOWNSCROLL       | Upscroll and downscroll
    REL_MISC                |    9     | XREL_MISC_1        | XREL_MISC_2       | Something?
    REL_RESERVED            |    10    | XREL_RESERVED_1    | XREL_RESERVED_2   | Something?
    REL_WHEEL_HI_RES        |    11    | XHIRES_UPSCROLL    | XHIRES_DOWNSCROLL | High resolution downscroll and upscroll, sent just after their non-high resolution version
    REL_HWHEEL_HI_RES       |    12    | XHIRES_RIGHTSCROLL | XHIRES_LEFTSCROLL | High resolution rightcroll and leftscroll, sent just after their non-high resolution version
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
              XRIGHTCURSOR: b
        "},
        vec![Event::RelativeEvent(
            get_input_device_info(),
            RelativeEvent::new_with(_REL_X, _POSITIVE),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

#[test]
fn verify_disguised_relative_events() {
    use crate::event_handler::DISGUISED_EVENT_OFFSETTER;
    // Verifies that the event offsetter used to "disguise" relative events into key event
    // is a bigger number than the biggest one a scancode had at the time of writing this (26 december 2022)
    assert!(0x2e7 < DISGUISED_EVENT_OFFSETTER);
    // and that it's not big enough that one of the "disguised" events's scancode would overflow.
    // (the largest of those events is equal to DISGUISED_EVENT_OFFSETTER + 26)
    assert!(DISGUISED_EVENT_OFFSETTER <= u16::MAX - 26)
}

#[test]
fn test_mouse_movement_event_accumulation() {
    // Tests that mouse movement events correctly get collected to be sent as one MouseMovementEventCollection,
    // which is necessary to avoid separating mouse movement events with synchronization events,
    // because such a separation would cause a bug with cursor movement.

    // Please refer to test_cursor_behavior_1 and test_cursor_behavior_2 for more information on said bug.
    assert_actions(
        indoc! {""},
        vec![
            Event::RelativeEvent(get_input_device_info(), RelativeEvent::new_with(_REL_X, _POSITIVE)),
            Event::RelativeEvent(get_input_device_info(), RelativeEvent::new_with(_REL_Y, _POSITIVE)),
        ],
        vec![Action::MouseMovementEventCollection(vec![
            RelativeEvent::new_with(_REL_X, _POSITIVE),
            RelativeEvent::new_with(_REL_Y, _POSITIVE),
        ])],
    )
}

#[test]
#[ignore]
// The OS interprets a REL_X event¹ combined with a REL_Y event² differently if they are separated by synchronization event.
// This test and test_cursor_behavior_2 are meant to be run to demonstrate that fact.

// ¹Mouse movement along the X (horizontal) axis.
// ²Mouse movement along the Y (vertical) axis.

// The only difference between test_cursor_behavior_1 and test_cursor_behavior_2 is that
// test_cursor_behavior_1 adds a synchronization event between REL_X and REL_Y events that would not normally be there.
// In other words, test_cursor_behavior_2 represents what would occur without Xremap intervention.

// Here's how to proceed :
// 1 - Move your mouse cursor to the bottom left of your screen.
// 2 - either run this test with sudo privileges or while your environnment is properly set up (https:// github.com/k0kubun/xremap#running-xremap-without-sudo),
//     so that your keyboard and/or mouse may be captured.

// 3 - Press any button (don't move the mouse).
// 4 - Note where the cursor ended up.

// 5 - Repeat steps 1 through 4 for test_cursor_behavior_2.
// 6 - Notice that the mouse cursor often ends up in a different position than when running test_cursor_behavior_1.

//
// Notes :
// - Because emitting an event automatcially adds a synchronization event afterwards (see https:// github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/uinput.rs#L167),
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
    // Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {}", e),
    };
    let mut output_device =
        match output_device(input_devices.values().next().map(InputDevice::bus_type), true, 0x1234, 0x5678) {
            Ok(output_device) => output_device,
            Err(e) => panic!("Failed to prepare an output device: {}", e),
        };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    // Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE.0, _REL_X, _POSITIVE),
                //
                // This line is the only difference between test_cursor_behavior_1 and test_cursor_behavior_2.
                InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
                //
                InputEvent::new_now(EventType::RELATIVE.0, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        // Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        // The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[ignore]
// The OS interprets a REL_X event combined with a REL_Y event differently if they are separated by synchronization event.
// This test and test_cursor_behavior_1 are meant to be run to demonstrate that fact.
// Please refer to the comment above test_cursor_behavior_1 for information on how to run these tests.
fn test_cursor_behavior_2() {
    use crate::device::InputDevice;
    use crate::device::{get_input_devices, output_device};
    // Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {}", e),
    };
    let mut output_device =
        match output_device(input_devices.values().next().map(InputDevice::bus_type), true, 0x1234, 0x5678) {
            Ok(output_device) => output_device,
            Err(e) => panic!("Failed to prepare an output device: {}", e),
        };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    // Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE.0, _REL_X, _POSITIVE),
                InputEvent::new_now(EventType::RELATIVE.0, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        // Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        // The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(Duration::from_millis(5));
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
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
        vec![Event::KeyEvent(
            get_input_device_info(),
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

    assert_actions_with_current_application(
        config,
        Some(String::from("firefox")),
        vec![Event::KeyEvent(
            get_input_device_info(),
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
            InputDeviceInfo {
                name: "Some Device",
                path: &Path::new("/dev/input/event0"),
                vendor: 0x1234,
                product: 0x5678,
            },
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
            InputDeviceInfo {
                name: "Other Device",
                path: &Path::new("/dev/input/event1"),
                vendor: 0x1234,
                product: 0x5678,
            },
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
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
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_keypress_and_remap_in_keymap_action() {
    // KEY_D will be emitted, and the remap will be used for next key press.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                - d
                - remap:
                    a: b
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_no_keypress_and_remap_in_keymap_action() {
    // The first match stops the search for matches. So the last remap isn't used.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
          - remap:
              f12:
                - remap:
                    a: b
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_no_keymap_action() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
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
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release))],
    )
}

#[test]
fn test_any_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a: b
              ANY: null
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );
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

    actual.append(&mut event_handler.on_events(&events, &config).unwrap());

    assert_eq!(format!("{:?}", actions), format!("{:?}", actual));
}
