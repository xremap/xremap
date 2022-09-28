use crate::client::{build_client, WMClient};
use crate::config::action::Action;
use crate::config::application::Application;
use crate::config::key_action::{KeyAction, MultiPurposeKey, PressReleaseKey};
use crate::config::key_press::{KeyPress, Modifier};
use crate::config::keymap::{build_override_table, OverrideEntry};
use crate::config::remap::Remap;
use crate::Config;
use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, InputEventKind, Key, RelativeAxisType};
use lazy_static::lazy_static;
use log::{debug, error};
use nix::sys::signal;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet};
use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{Expiration, TimerFd, TimerSetTimeFlags};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct MappableEvent {
    key: Key,
    value: i32,
    raw: InputEvent,
}

impl MappableEvent {
    pub fn new(key: Key, value: i32) -> Self {
        let raw = InputEvent::new(EventType::KEY, key.code(), value);
        Self { key, value, raw }
    }

    pub fn from(raw: InputEvent) -> Option<Self> {
        match raw.kind() {
            InputEventKind::Key(key) => Some(Self {
                key,
                value: raw.value(),
                raw,
            }),
            // Scrollwheel events are treated as if they were SCROLLUP/SCROLLDOWN keypresses,
            // but we emit the original event if it doesn't get remapped.
            InputEventKind::RelAxis(RelativeAxisType::REL_WHEEL) => {
                let key = match raw.value() {
                    1 => Key::KEY_SCROLLUP,
                    -1 => Key::KEY_SCROLLDOWN,
                    other => {
                        debug!("Unknown scroll value: {}", other);
                        return None;
                    }
                };
                Some(Self { key, value: PRESS, raw })
            }
            _ => None,
        }
    }
}

pub struct EventHandler {
    // Device to emit events
    device: VirtualDevice,
    // Currently pressed modifier keys
    modifiers: HashSet<Key>,
    // Modifiers that are currently pressed but not in the source KeyPress
    extra_modifiers: HashSet<Key>,
    // Make sure the original event is released even if remapping changes while holding the key
    pressed_keys: HashMap<Key, Key>,
    // Check the currently active application
    application_client: WMClient,
    application_cache: Option<String>,
    // State machine for multi-purpose keys
    multi_purpose_keys: HashMap<Key, MultiPurposeKeyState>,
    // Current nested remaps
    override_remap: Option<HashMap<Key, Vec<OverrideEntry>>>,
    // Key triggered on a timeout of nested remaps
    override_timeout_key: Option<Key>,
    // Trigger a timeout of nested remaps through select(2)
    override_timer: TimerFd,
    // Whether we've called a sigaction for spawing commands or not
    sigaction_set: bool,
    // { set_mode: String }
    mode: String,
    // { set_mark: true }
    mark_set: bool,
    // { escape_next_key: true }
    escape_next_key: bool,
}

impl EventHandler {
    pub fn new(device: VirtualDevice, timer: TimerFd, mode: &str) -> EventHandler {
        EventHandler {
            device,
            modifiers: HashSet::new(),
            extra_modifiers: HashSet::new(),
            pressed_keys: HashMap::new(),
            application_client: build_client(),
            application_cache: None,
            multi_purpose_keys: HashMap::new(),
            override_remap: None,
            override_timeout_key: None,
            override_timer: timer,
            sigaction_set: false,
            mode: mode.to_string(),
            mark_set: false,
            escape_next_key: false,
        }
    }

    pub fn on_event(&mut self, event: InputEvent, config: &Config) -> Result<(), Box<dyn Error>> {
        if let Some(mappable) = MappableEvent::from(event) {
            self.on_mappable_event(mappable, config)
        } else {
            Ok(self.send_event(event)?)
        }
    }

    fn on_mappable_event(&mut self, event: MappableEvent, config: &Config) -> Result<(), Box<dyn Error>> {
        self.application_cache = None; // expire cache
        debug!("=> {}: {:?}", event.value, &event.key);

        // Apply modmap
        let mut events = if let Some(key_action) = self.find_modmap(config, &event.key) {
            self.dispatch_keys(key_action, event)?
        } else {
            vec![event]
        };
        self.maintain_pressed_keys(event.key, event.value, &mut events);
        if !self.multi_purpose_keys.is_empty() {
            events = self.flush_timeout_keys(events);
        }

        // Apply keymap
        for event in events.into_iter() {
            if config.virtual_modifiers.contains(&event.key) {
                self.update_modifier(event);
                continue;
            } else if MODIFIER_KEYS.contains(&event.key) {
                self.update_modifier(event);
            } else if is_pressed(event.value) {
                if self.escape_next_key {
                    self.escape_next_key = false
                } else if let Some(actions) = self.find_keymap(config, &event.key)? {
                    self.dispatch_actions(&actions, &event.key)?;
                    continue;
                }
            }
            self.send_event(event.raw)?;
        }
        Ok(())
    }

    pub fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        if event.event_type() == EventType::KEY {
            debug!("{}: {:?}", event.value(), Key::new(event.code()))
        }
        self.device.emit(&[event])
    }

    pub fn timeout_override(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(key) = self.override_timeout_key {
            self.send_key(&key, PRESS)?;
            self.send_key(&key, RELEASE)?;
        }
        self.remove_override()
    }

    fn remove_override(&mut self) -> Result<(), Box<dyn Error>> {
        self.override_timer.unset()?;
        self.override_remap = None;
        self.override_timeout_key = None;
        Ok(())
    }

    fn send_keys(&mut self, keys: &Vec<Key>, value: i32) -> std::io::Result<()> {
        for key in keys {
            self.send_key(key, value)?;
        }
        Ok(())
    }

    fn send_key(&mut self, key: &Key, value: i32) -> std::io::Result<()> {
        self.send_event(InputEvent::new(EventType::KEY, key.code(), value))
    }

    // Repeat/Release what's originally pressed even if remapping changes while holding it
    fn maintain_pressed_keys(&mut self, key: Key, value: i32, events: &mut Vec<MappableEvent>) {
        // Not handling multi-purpose keysfor now; too complicated
        if events.len() != 1 || value != events[0].value {
            return;
        }

        let event = events[0];
        if event.value == PRESS {
            self.pressed_keys.insert(key, event.key);
        } else {
            if let Some(original_key) = self.pressed_keys.get(&key) {
                events[0].key = *original_key;
            }
            if value == RELEASE {
                self.pressed_keys.remove(&key);
            }
        }
    }

    fn dispatch_keys(
        &mut self,
        key_action: KeyAction,
        event: MappableEvent,
    ) -> Result<Vec<MappableEvent>, Box<dyn Error>> {
        let keys = match key_action {
            KeyAction::Key(modmap_key) => vec![MappableEvent::new(modmap_key, event.value)],
            KeyAction::MultiPurposeKey(MultiPurposeKey {
                held,
                alone,
                alone_timeout,
            }) => {
                if event.value == PRESS {
                    self.multi_purpose_keys.insert(
                        event.key,
                        MultiPurposeKeyState {
                            held,
                            alone,
                            alone_timeout_at: Some(Instant::now() + alone_timeout),
                        },
                    );
                    return Ok(vec![]); // delay the press
                } else if event.value == REPEAT {
                    if let Some(state) = self.multi_purpose_keys.get_mut(&event.key) {
                        return Ok(state.repeat());
                    }
                } else if event.value == RELEASE {
                    if let Some(state) = self.multi_purpose_keys.remove(&event.key) {
                        return Ok(state.release());
                    }
                } else {
                    panic!("unexpected key event value: {}", event.value);
                }
                // fallthrough on state discrepancy
                vec![event]
            }
            KeyAction::PressReleaseKey(PressReleaseKey { press, release }) => {
                // Just hook actions, and then emit the original event. We might want to
                // support reordering the key event and dispatched actions later.
                if event.value == PRESS {
                    self.dispatch_actions(&press, &event.key)?;
                }
                if event.value == RELEASE {
                    self.dispatch_actions(&release, &event.key)?;
                }
                // Dispatch the original key as well
                vec![event]
            }
        };
        Ok(keys)
    }

    fn flush_timeout_keys(&mut self, events: Vec<MappableEvent>) -> Vec<MappableEvent> {
        let mut flush = false;
        for event in events.iter() {
            if event.value == PRESS {
                flush = true;
                break;
            }
        }

        if flush {
            let mut flushed: Vec<MappableEvent> = vec![];
            for (_, state) in self.multi_purpose_keys.iter_mut() {
                flushed.extend(state.force_held());
            }
            flushed.extend(events);
            flushed
        } else {
            events
        }
    }

    fn find_modmap(&mut self, config: &Config, key: &Key) -> Option<KeyAction> {
        for modmap in &config.modmap {
            if let Some(key_action) = modmap.remap.get(key) {
                if let Some(application_matcher) = &modmap.application {
                    if !self.match_application(application_matcher) {
                        continue;
                    }
                }
                return Some(key_action.clone());
            }
        }
        None
    }

    fn find_keymap(&mut self, config: &Config, key: &Key) -> Result<Option<Vec<Action>>, Box<dyn Error>> {
        if let Some(override_remap) = &self.override_remap {
            if let Some(entries) = override_remap.clone().get(key) {
                self.remove_override()?;
                for exact_match in [true, false] {
                    for entry in entries {
                        let (extra_modifiers, missing_modifiers) = self.diff_modifiers(&entry.modifiers);
                        if (exact_match && extra_modifiers.len() > 0) || missing_modifiers.len() > 0 {
                            continue;
                        }
                        return Ok(Some(with_extra_modifiers(&entry.actions, &extra_modifiers)));
                    }
                }
            }
        } else {
            self.timeout_override()?;
        }
        if let Some(entries) = config.keymap_table.get(key) {
            for exact_match in [true, false] {
                for entry in entries {
                    let (extra_modifiers, missing_modifiers) = self.diff_modifiers(&entry.modifiers);
                    if (exact_match && extra_modifiers.len() > 0) || missing_modifiers.len() > 0 {
                        continue;
                    }
                    if let Some(application_matcher) = &entry.application {
                        if !self.match_application(application_matcher) {
                            continue;
                        }
                    }
                    if let Some(modes) = &entry.mode {
                        if !modes.contains(&self.mode) {
                            continue;
                        }
                    }
                    return Ok(Some(with_extra_modifiers(&entry.actions, &extra_modifiers)));
                }
            }
        }
        Ok(None)
    }

    fn dispatch_actions(&mut self, actions: &Vec<Action>, key: &Key) -> Result<(), Box<dyn Error>> {
        for action in actions {
            self.dispatch_action(action, key)?;
        }
        Ok(())
    }

    fn dispatch_action(&mut self, action: &Action, key: &Key) -> Result<(), Box<dyn Error>> {
        match action {
            Action::KeyPress(key_press) => self.send_key_press(key_press)?,
            Action::Remap(Remap {
                remap,
                timeout,
                timeout_key,
            }) => {
                self.override_remap = Some(build_override_table(remap));
                if let Some(timeout) = timeout {
                    let expiration = Expiration::OneShot(TimeSpec::from_duration(*timeout));
                    self.override_timer.unset()?;
                    self.override_timer.set(expiration, TimerSetTimeFlags::empty())?;
                    self.override_timeout_key = timeout_key.or_else(|| Some(*key));
                }
            }
            Action::Launch(command) => self.run_command(command.clone()),
            Action::SetMode(mode) => {
                self.mode = mode.clone();
                println!("mode: {}", mode);
            }
            Action::SetMark(set) => self.mark_set = *set,
            Action::WithMark(key_press) => self.send_key_press(&self.with_mark(key_press))?,
            Action::EscapeNextKey(escape_next_key) => self.escape_next_key = *escape_next_key,
            Action::SetExtraModifiers(keys) => {
                self.extra_modifiers.clear();
                for key in keys {
                    self.extra_modifiers.insert(*key);
                }
            }
        }
        Ok(())
    }

    fn send_key_press(&mut self, key_press: &KeyPress) -> Result<(), Box<dyn Error>> {
        // Build extra or missing modifiers. Note that only MODIFIER_KEYS are handled
        // because logical modifiers shouldn't make an impact outside xremap.
        let (mut extra_modifiers, mut missing_modifiers) = self.diff_modifiers(&key_press.modifiers);
        extra_modifiers.retain(|key| MODIFIER_KEYS.contains(&key) && !self.extra_modifiers.contains(&key));
        missing_modifiers.retain(|key| MODIFIER_KEYS.contains(&key));

        // Emulate the modifiers of KeyPress
        self.send_keys(&extra_modifiers, RELEASE)?;
        self.send_keys(&missing_modifiers, PRESS)?;

        // Press the main key
        self.send_key(&key_press.key, PRESS)?;
        self.send_key(&key_press.key, RELEASE)?;

        // Resurrect the original modifiers
        self.send_keys(&missing_modifiers, RELEASE)?;
        self.send_keys(&extra_modifiers, PRESS)?;

        Ok(())
    }

    fn with_mark(&self, key_press: &KeyPress) -> KeyPress {
        if self.mark_set && !self.match_modifier(&Modifier::Shift) {
            let mut modifiers = key_press.modifiers.clone();
            modifiers.push(Modifier::Shift);
            KeyPress {
                key: key_press.key,
                modifiers,
            }
        } else {
            key_press.clone()
        }
    }

    fn run_command(&mut self, command: Vec<String>) {
        if !self.sigaction_set {
            // Avoid defunct processes
            let sig_action = SigAction::new(SigHandler::SigDfl, SaFlags::SA_NOCLDWAIT, SigSet::empty());
            unsafe {
                sigaction(signal::SIGCHLD, &sig_action).expect("Failed to register SIGCHLD handler");
            }
            self.sigaction_set = true;
        }

        debug!("Running command: {:?}", command);
        match Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => debug!("Process spawned: {:?}, pid {}", command, child.id()),
            Err(e) => error!("Error running command: {:?}", e),
        }
    }

    // Return (extra_modifiers, missing_modifiers)
    fn diff_modifiers(&self, modifiers: &Vec<Modifier>) -> (Vec<Key>, Vec<Key>) {
        let extra_modifiers: Vec<Key> = self
            .modifiers
            .iter()
            .filter(|modifier| !contains_modifier(modifiers, modifier))
            .map(|modifier| modifier.clone())
            .collect();
        let missing_modifiers: Vec<Key> = modifiers
            .iter()
            .filter_map(|modifier| {
                if self.match_modifier(modifier) {
                    None
                } else {
                    match modifier {
                        Modifier::Shift => Some(Key::KEY_LEFTSHIFT),
                        Modifier::Control => Some(Key::KEY_LEFTCTRL),
                        Modifier::Alt => Some(Key::KEY_LEFTALT),
                        Modifier::Windows => Some(Key::KEY_LEFTMETA),
                        Modifier::Key(key) => Some(*key),
                    }
                }
            })
            .collect();
        return (extra_modifiers, missing_modifiers);
    }

    fn match_modifier(&self, modifier: &Modifier) -> bool {
        match modifier {
            Modifier::Shift => {
                self.modifiers.contains(&Key::KEY_LEFTSHIFT) || self.modifiers.contains(&Key::KEY_RIGHTSHIFT)
            }
            Modifier::Control => {
                self.modifiers.contains(&Key::KEY_LEFTCTRL) || self.modifiers.contains(&Key::KEY_RIGHTCTRL)
            }
            Modifier::Alt => self.modifiers.contains(&Key::KEY_LEFTALT) || self.modifiers.contains(&Key::KEY_RIGHTALT),
            Modifier::Windows => {
                self.modifiers.contains(&Key::KEY_LEFTMETA) || self.modifiers.contains(&Key::KEY_RIGHTMETA)
            }
            Modifier::Key(key) => self.modifiers.contains(key),
        }
    }

    fn match_application(&mut self, application_matcher: &Application) -> bool {
        // Lazily fill the wm_class cache
        if self.application_cache.is_none() {
            match self.application_client.current_application() {
                Some(application) => self.application_cache = Some(application),
                None => self.application_cache = Some(String::new()),
            }
        }

        if let Some(application) = &self.application_cache {
            if let Some(application_only) = &application_matcher.only {
                return application_only.iter().any(|m| m.matches(application));
            }
            if let Some(application_not) = &application_matcher.not {
                return application_not.iter().all(|m| !m.matches(application));
            }
        }
        false
    }

    fn update_modifier(&mut self, event: MappableEvent) {
        if event.value == PRESS {
            self.modifiers.insert(event.key);
        } else if event.value == RELEASE {
            self.modifiers.remove(&event.key);
        }
    }
}

fn with_extra_modifiers(actions: &Vec<Action>, extra_modifiers: &Vec<Key>) -> Vec<Action> {
    let mut result: Vec<Action> = vec![];
    if extra_modifiers.len() > 0 {
        // Virtually release extra modifiers so that they won't be physically released on KeyPress
        result.push(Action::SetExtraModifiers(extra_modifiers.clone()));
    }
    result.extend(actions.clone());
    if extra_modifiers.len() > 0 {
        // Resurrect the modifier status
        result.push(Action::SetExtraModifiers(vec![]));
    }
    return result;
}

fn contains_modifier(modifiers: &Vec<Modifier>, key: &Key) -> bool {
    for modifier in modifiers {
        if match modifier {
            Modifier::Shift => key == &Key::KEY_LEFTSHIFT || key == &Key::KEY_RIGHTSHIFT,
            Modifier::Control => key == &Key::KEY_LEFTCTRL || key == &Key::KEY_RIGHTCTRL,
            Modifier::Alt => key == &Key::KEY_LEFTALT || key == &Key::KEY_RIGHTALT,
            Modifier::Windows => key == &Key::KEY_LEFTMETA || key == &Key::KEY_RIGHTMETA,
            Modifier::Key(modifier_key) => key == modifier_key,
        } {
            return true;
        }
    }
    false
}

lazy_static! {
    static ref MODIFIER_KEYS: [Key; 8] = [
        // Shift
        Key::KEY_LEFTSHIFT,
        Key::KEY_RIGHTSHIFT,
        // Control
        Key::KEY_LEFTCTRL,
        Key::KEY_RIGHTCTRL,
        // Alt
        Key::KEY_LEFTALT,
        Key::KEY_RIGHTALT,
        // Windows
        Key::KEY_LEFTMETA,
        Key::KEY_RIGHTMETA,
    ];
}

//---

fn is_pressed(value: i32) -> bool {
    value == PRESS || value == REPEAT
}

// InputEvent#value
static RELEASE: i32 = 0;
static PRESS: i32 = 1;
static REPEAT: i32 = 2;

//---

#[derive(Debug)]
struct MultiPurposeKeyState {
    held: Key,
    alone: Key,
    // Some if the first press is still delayed, None if already considered held.
    alone_timeout_at: Option<Instant>,
}

impl MultiPurposeKeyState {
    fn repeat(&mut self) -> Vec<MappableEvent> {
        if let Some(alone_timeout_at) = &self.alone_timeout_at {
            if Instant::now() < *alone_timeout_at {
                vec![] // still delay the press
            } else {
                self.alone_timeout_at = None; // timeout
                vec![MappableEvent::new(self.held, PRESS)]
            }
        } else {
            vec![MappableEvent::new(self.held, REPEAT)]
        }
    }

    fn release(&self) -> Vec<MappableEvent> {
        if let Some(alone_timeout_at) = &self.alone_timeout_at {
            if Instant::now() < *alone_timeout_at {
                // dispatch the delayed press and this release
                vec![
                    MappableEvent::new(self.alone, PRESS),
                    MappableEvent::new(self.alone, RELEASE),
                ]
            } else {
                // too late. dispatch the held key
                vec![
                    MappableEvent::new(self.held, PRESS),
                    MappableEvent::new(self.held, RELEASE),
                ]
            }
        } else {
            vec![MappableEvent::new(self.held, RELEASE)]
        }
    }

    fn force_held(&mut self) -> Vec<MappableEvent> {
        if self.alone_timeout_at.is_some() {
            self.alone_timeout_at = None;
            vec![MappableEvent::new(self.held, PRESS)]
        } else {
            vec![]
        }
    }
}
