#![cfg(test)]

use evdev::KeyCode as Key;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::path::Path;
use std::time::Duration;

use crate::action::Action;
use crate::client::Client;
use crate::client::WMClient;
use crate::config::keymap::build_keymap_table;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::EventHandler;
use crate::Config;
use serde::Serialize;
use std::collections::BTreeMap;

struct StaticClient;
impl Client for StaticClient {
    fn supported(&mut self) -> bool {
        true
    }
    fn current_window(&mut self) -> Option<String> {
        None
    }
    fn current_application(&mut self) -> Option<String> {
        None
    }
}

enum Phase {
    Events(Vec<(Key, KeyValue)>),
    Sleep(u64),
}

const KEY_SPACE: Key = Key::KEY_SPACE;
const KEY_A: Key = Key::KEY_A;
const KEY_LEFTSHIFT: Key = Key::KEY_LEFTSHIFT;

trait KeyTupleExt {
    fn press(self) -> (Key, KeyValue);
    fn release(self) -> (Key, KeyValue);

    // NOTE: KeyValue::Repeat corresponds to the kernel's auto-repeat event fired when a key is held until after `repeat_delay`
    //
    // NOTE:
    // since We’re not streaming repeats like the kernel would.
    // we need need to raw dog “still down” signal to advance the state machine
    fn repeat(self) -> (Key, KeyValue);
    // i guess press + release could be wrapped an `alone`
    // and press + repeat as `held`
    // but this implementation makes testing for phased events simpler. (when alone_timeout_millis,
    // modifies the event chain)
    //
    // check our phased_default() test for an example
}

impl KeyTupleExt for Key {
    fn press(self) -> (Key, KeyValue) {
        (self, KeyValue::Press)
    }
    fn release(self) -> (Key, KeyValue) {
        (self, KeyValue::Release)
    }
    fn repeat(self) -> (Key, KeyValue) {
        (self, KeyValue::Repeat)
    }
}

struct TestCtx {
    handler: EventHandler,
    config: Config,
}

impl TestCtx {
    // fn new() -> Self {
    //     Self {
    //         handler: make_handler(),
    //         config: build_config_opts(None, None),
    //     }
    // }

    fn new(timeout_ms: Option<u64>, free_hold: Option<bool>) -> Self {
        Self {
            handler: mk_handler(),
            config: config_opts(timeout_ms, free_hold),
        }
    }

    // fn with_events(&mut self, events: Vec<Event<'static>>, config: &Config) -> Vec<Action> {
    //     self.handler.on_events(&events, config).expect("handler")
    // }

    fn run(&mut self, res_set: Vec<(Key, KeyValue)>) -> Vec<Action> {
        let events = evs(res_set);
        self.handler.on_events(&events, &self.config).expect("handler")
    }

    fn run_and_assert(&mut self, res_set: Vec<(Key, KeyValue)>, expected_set: Vec<(Key, KeyValue)>) {
        let actions = self.run(res_set);
        let expected = expected(expected_set);
        assert_eq!(format!("{:?}", expected), format!("{:?}", actions));
    }

    fn run_phases_and_assert(&mut self, phases: Vec<Phase>, expected_set: Vec<(Key, KeyValue)>) {
        let mut actions: Vec<Action> = Vec::new();
        for phase in phases {
            match phase {
                Phase::Events(res_set) => actions.extend(self.run(res_set)),
                Phase::Sleep(ms) => std::thread::sleep(Duration::from_millis(ms)),
            }
        }
        let expected = expected(expected_set);
        assert_eq!(format!("{:?}", expected), format!("{:?}", actions));
    }
}

fn dev_info() -> InputDeviceInfo<'static> {
    InputDeviceInfo {
        name: "TestDev",
        path: Path::new("/dev/input/event0"),
        vendor: 0x1,
        product: 0x1,
    }
}

fn config_opts(timeout_ms: Option<u64>, free_hold: Option<bool>) -> Config {
    #[derive(Serialize)]
    struct RemapEntry<'a> {
        held: &'a str,
        alone: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        alone_timeout_millis: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        free_hold: Option<bool>,
    }

    #[derive(Serialize)]
    struct ModMapItem<'a> {
        name: &'a str,
        remap: BTreeMap<&'a str, RemapEntry<'a>>,
    }

    #[derive(Serialize)]
    struct Root<'a> {
        modmap: Vec<ModMapItem<'a>>,
    }

    let mut remap = BTreeMap::new();
    remap.insert(
        "Space",
        RemapEntry {
            held: "Shift_L",
            alone: "Space",
            alone_timeout_millis: timeout_ms,
            free_hold,
        },
    );

    let root = Root {
        modmap: vec![ModMapItem { name: "test", remap }],
    };

    let yaml = serde_yaml::to_string(&root).unwrap();

    let mut config: Config = serde_yaml::from_str(&yaml).unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    config
}

fn timerfd() -> TimerFd {
    TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).expect("timerfd")
}

fn mk_handler() -> EventHandler {
    let client = WMClient::new("static", Box::new(StaticClient));
    EventHandler::new(timerfd(), "default", Duration::ZERO, client)
}

fn evs<I>(set: I) -> Vec<Event<'static>>
where
    I: IntoIterator<Item = (Key, KeyValue)>,
{
    let ev = |k: Key, v: KeyValue| Event::KeyEvent(dev_info(), KeyEvent::new(k, v));
    set.into_iter().map(|(k, v)| ev(k, v)).collect()
}

fn expected<I>(set: I) -> Vec<Action>
where
    I: IntoIterator<Item = (Key, KeyValue)>,
{
    set.into_iter()
        .map(|(k, v)| Action::KeyEvent(KeyEvent::new(k, v)))
        .collect()
}

#[test]
fn free_hold_combo() {
    let mut ctx = TestCtx::new(Some(400), Some(true));
    ctx.run_and_assert(
        vec![KEY_SPACE.press(), KEY_A.press(), KEY_A.release(), KEY_SPACE.release()],
        vec![
            KEY_LEFTSHIFT.press(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_LEFTSHIFT.release(),
        ],
    );
}

#[test]
fn within_window_combo_no_stuck_shift() {
    let mut ctx = TestCtx::new(Some(400), Some(false));

    ctx.run_and_assert(
        vec![KEY_SPACE.press(), KEY_A.press(), KEY_A.release(), KEY_SPACE.release()],
        vec![
            KEY_LEFTSHIFT.press(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_LEFTSHIFT.release(),
        ],
    );
}

#[test]
fn free_hold_combo_default_timeout() {
    let mut ctx = TestCtx::new(None, Some(true));
    ctx.run_and_assert(
        vec![KEY_SPACE.press(), KEY_A.press(), KEY_A.release(), KEY_SPACE.release()],
        vec![
            KEY_LEFTSHIFT.press(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_LEFTSHIFT.release(),
        ],
    );
}

// NOTE: setting alone_timeout_ms to 0  forces the key to transition to the "held" state
// on the very next event after the initial press!
// this lets us advance the state machine deterministically.
// could be useful to avoid phasing in very rare cases when real-time sleep flakiness is a concern

#[test]
fn split_phased_default() {
    let mut ctx = TestCtx::new(Some(0), Some(false));
    ctx.run_and_assert(
        vec![
            KEY_SPACE.press(),
            KEY_SPACE.repeat(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_SPACE.release(),
        ],
        vec![
            KEY_LEFTSHIFT.press(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_LEFTSHIFT.release(),
        ],
    );
}

#[test]
fn phased_default() {
    let mut ctx = TestCtx::new(Some(400), None);

    ctx.run_phases_and_assert(
        vec![
            Phase::Events(vec![KEY_SPACE.press()]),
            Phase::Sleep(450),
            Phase::Events(vec![KEY_SPACE.repeat()]),
        ],
        vec![KEY_LEFTSHIFT.press()],
    );
}

// NOTE: this would've failed and catched https://github.com/xremap/xremap/discussions/724
#[test]
fn after_window_phased() {
    let mut ctx = TestCtx::new(Some(400), Some(false));

    ctx.run_phases_and_assert(
        vec![
            Phase::Events(vec![KEY_SPACE.press()]),
            Phase::Sleep(450),
            Phase::Events(vec![KEY_A.press(), KEY_A.release(), KEY_SPACE.release()]),
        ],
        vec![
            KEY_LEFTSHIFT.press(),
            KEY_A.press(),
            KEY_A.release(),
            KEY_LEFTSHIFT.release(),
        ],
    );
}

#[test]
fn phased_default_within_window() {
    let mut ctx = TestCtx::new(Some(400), None);

    ctx.run_phases_and_assert(
        vec![
            Phase::Events(vec![KEY_SPACE.press()]),
            Phase::Sleep(250),
            Phase::Events(vec![KEY_SPACE.release()]),
        ],
        vec![KEY_SPACE.press(), KEY_SPACE.release()],
    );
}

#[test]
fn phased_default_free_hold() {
    let mut ctx = TestCtx::new(Some(400), Some(true));

    ctx.run_phases_and_assert(
        vec![
            Phase::Events(vec![KEY_SPACE.press()]),
            Phase::Sleep(450),
            Phase::Events(vec![KEY_SPACE.release()]),
        ],
        vec![KEY_SPACE.press(), KEY_SPACE.release()],
    );
}
