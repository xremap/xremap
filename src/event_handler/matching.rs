use evdev::KeyCode as Key;

use crate::config::keymap_action::KeymapAction;
use crate::config::modmap_action::ModmapAction;
use crate::config::application::OnlyOrNot;
use crate::device::InputDeviceInfo;
use crate::{config, Config};
use crate::config::keymap::OverrideEntry;
use crate::config::key_press::Modifier;

use super::TaggedAction;
use super::EventHandler;

pub(super) fn contains_modifier(modifiers: &[Modifier], key: &Key) -> bool {
    for modifier in modifiers {
        let matches = match modifier {
            Modifier::Shift => key == &Key::KEY_LEFTSHIFT || key == &Key::KEY_RIGHTSHIFT,
            Modifier::Control => key == &Key::KEY_LEFTCTRL || key == &Key::KEY_RIGHTCTRL,
            Modifier::Alt => key == &Key::KEY_LEFTALT || key == &Key::KEY_RIGHTALT,
            Modifier::Windows => key == &Key::KEY_LEFTMETA || key == &Key::KEY_RIGHTMETA,
            Modifier::Key(modifier_key) => key == modifier_key,
        };
        if matches {
            return true;
        }
    }
    false
}

impl EventHandler {
    // Return (extra_modifiers, missing_modifiers)
    pub(super) fn diff_modifiers(&self, modifiers: &[Modifier]) -> (Vec<Key>, Vec<Key>) {
        let extra_modifiers: Vec<Key> = self
            .modifiers
            .iter()
            .filter(|modifier| !contains_modifier(modifiers, modifier))
            .copied()
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
        (extra_modifiers, missing_modifiers)
    }

    pub(super) fn match_modifier(&self, modifier: &Modifier) -> bool {
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

    pub(super) fn match_window(&mut self, window_matcher: &OnlyOrNot) -> bool {
        // Lazily fill the wm_class cache
        if self.title_cache.is_none() {
            match self.application_client.current_window() {
                Some(title) => self.title_cache = Some(title),
                None => self.title_cache = Some(String::new()),
            }
        }

        if let Some(title) = &self.title_cache {
            if let Some(title_only) = &window_matcher.only {
                return title_only.iter().any(|m| m.matches(title));
            }
            if let Some(title_not) = &window_matcher.not {
                return title_not.iter().all(|m| !m.matches(title));
            }
        }
        false
    }

    pub(super) fn match_application(&mut self, application_matcher: &OnlyOrNot) -> bool {
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

    pub(super) fn match_device(&self, device_matcher: &config::device::Device, device: &InputDeviceInfo) -> bool {
        if let Some(device_only) = &device_matcher.only {
            return device_only.iter().any(|m| device.matches(m));
        }
        if let Some(device_not) = &device_matcher.not {
            return device_not.iter().all(|m| !device.matches(m));
        }
        false
    }

    pub(super) fn find_modmap(&mut self, config: &Config, key: &Key, device: &InputDeviceInfo) -> Option<ModmapAction> {
        for modmap in &config.modmap {
            if let Some(key_action) = modmap.remap.get(key) {
                if let Some(window_matcher) = &modmap.window {
                    if !self.match_window(window_matcher) {
                        continue;
                    }
                }
                if let Some(application_matcher) = &modmap.application {
                    if !self.match_application(application_matcher) {
                        continue;
                    }
                }
                if let Some(device_matcher) = &modmap.device {
                    if !self.match_device(device_matcher, device) {
                        continue;
                    }
                }
                if let Some(modes) = &modmap.mode {
                    if !modes.contains(&self.mode) {
                        continue;
                    }
                }
                return Some(key_action.clone());
            }
        }
        None
    }

    pub(super) fn find_keymap(
        &mut self,
        config: &Config,
        key: &Key,
        device: &InputDeviceInfo,
    ) -> Result<Option<Vec<TaggedAction>>, Box<dyn std::error::Error>> {
        if !self.override_remaps.is_empty() {
            let entries: Vec<OverrideEntry> = self
                .override_remaps
                .iter()
                .flat_map(|map| map.get(key).cloned().unwrap_or_default())
                .collect();

            if !entries.is_empty() {
                self.remove_override()?;

                for exact_match in [true, false] {
                    let mut remaps = vec![];
                    for entry in &entries {
                        if entry.exact_match && !exact_match {
                            continue;
                        }
                        let (extra_modifiers, missing_modifiers) = self.diff_modifiers(&entry.modifiers);
                        if (exact_match && !extra_modifiers.is_empty()) || !missing_modifiers.is_empty() {
                            continue;
                        }

                        let actions = with_extra_modifiers(&entry.actions, &extra_modifiers, entry.exact_match);
                        let is_remap = is_remap(&entry.actions);

                        // If the first/top match was a remap, continue to find rest of the eligible remaps for this key
                        if remaps.is_empty() && !is_remap {
                            return Ok(Some(actions));
                        } else if is_remap {
                            remaps.extend(actions);
                        }
                    }
                    if !remaps.is_empty() {
                        return Ok(Some(remaps));
                    }
                }
            }
            // An override remap is set but not used. Flush the pending key.
            self.timeout_override()?;
        }

        if let Some(entries) = config.keymap_table.get(key) {
            for exact_match in [true, false] {
                let mut remaps = vec![];
                for entry in entries {
                    if entry.exact_match && !exact_match {
                        continue;
                    }
                    let (extra_modifiers, missing_modifiers) = self.diff_modifiers(&entry.modifiers);
                    if (exact_match && !extra_modifiers.is_empty()) || !missing_modifiers.is_empty() {
                        continue;
                    }
                    if let Some(window_matcher) = &entry.title {
                        if !self.match_window(window_matcher) {
                            continue;
                        }
                    }

                    if let Some(application_matcher) = &entry.application {
                        if !self.match_application(application_matcher) {
                            continue;
                        }
                    }
                    if let Some(device_matcher) = &entry.device {
                        if !self.match_device(device_matcher, device) {
                            continue;
                        }
                    }
                    if let Some(modes) = &entry.mode {
                        if !modes.contains(&self.mode) {
                            continue;
                        }
                    }

                    let actions = with_extra_modifiers(&entry.actions, &extra_modifiers, entry.exact_match);
                    let is_remap = is_remap(&entry.actions);

                    // If the first/top match was a remap, continue to find rest of the eligible remaps for this key
                    if remaps.is_empty() && !is_remap {
                        return Ok(Some(actions));
                    } else if is_remap {
                        remaps.extend(actions)
                    }
                }
                if !remaps.is_empty() {
                    return Ok(Some(remaps));
                }
            }
        }
        Ok(None)
    }
}

pub(super) fn is_remap(actions: &[KeymapAction]) -> bool {
    if actions.is_empty() {
        return false;
    }
    actions.iter().all(|x| matches!(x, KeymapAction::Remap(..)))
}

pub(super) fn with_extra_modifiers(
    actions: &[KeymapAction],
    extra_modifiers: &[Key],
    exact_match: bool,
) -> Vec<TaggedAction> {
    let mut result: Vec<TaggedAction> = vec![];
    if !extra_modifiers.is_empty() {
        // Virtually release extra modifiers so that they won't be physically released on KeyPress
        result.push(TaggedAction {
            action: KeymapAction::SetExtraModifiers(extra_modifiers.to_vec()),
            exact_match,
        });
    }
    result.extend(actions.iter().map(|action| TaggedAction {
        action: action.clone(),
        exact_match,
    }));
    if !extra_modifiers.is_empty() {
        // Resurrect the modifier status
        result.push(TaggedAction {
            action: KeymapAction::SetExtraModifiers(vec![]),
            exact_match,
        });
    }
    result
}
