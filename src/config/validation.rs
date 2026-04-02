use crate::config::key_press::KeyPress;
use crate::config::keymap_action::KeymapAction;
use crate::config::modmap_operator::ModmapOperator;
use crate::config::Config;
use crate::event_handler::{DISGUISED_EVENT_OFFSETTER, KEY_MATCH_ANY};
use anyhow::bail;
use evdev::KeyCode as Key;
use std::collections::HashMap;

pub fn validate_config_file(config: &Config) -> anyhow::Result<()> {
    for modmap in &config.modmap {
        for (key, operator) in &modmap.remap {
            traverse_modmap_keys(&vec![*key])?;
            traverse_modmap_operator(operator)?;
        }
    }

    for keymap in &config.keymap {
        traverse_remap(&keymap.remap)?;
    }

    Ok(())
}

fn traverse_modmap_keys(keys: &Vec<Key>) -> anyhow::Result<()> {
    for key in keys {
        if *key == KEY_MATCH_ANY {
            bail!("Any-key can't be used in modmap")
        }
        if key.code() >= DISGUISED_EVENT_OFFSETTER {
            bail!("Relative mouse events can't be used in modmap")
        }
    }

    Ok(())
}

fn traverse_keymap_output_keys(keys: &Vec<Key>) -> anyhow::Result<()> {
    for key in keys {
        if *key == KEY_MATCH_ANY {
            bail!("Any-key can't be used as output")
        }
        if key.code() >= DISGUISED_EVENT_OFFSETTER {
            bail!("Relative mouse events can't be used as output")
        }
    }

    Ok(())
}

fn traverse_modmap_operator(operator: &ModmapOperator) -> anyhow::Result<()> {
    match operator {
        ModmapOperator::Keys(keys) => {
            traverse_modmap_keys(&keys.clone().into_vec())?;
        }
        ModmapOperator::MultiPurposeKey(multi_purpose_key) => {
            traverse_modmap_keys(&multi_purpose_key.tap.clone().into_vec())?;
            traverse_modmap_keys(&multi_purpose_key.hold.clone().into_vec())?;
        }
        ModmapOperator::PressReleaseKey(operator) => {
            // This is keymap actions, though in modmap.
            traverse_actions(&operator.press)?;
            traverse_actions(&operator.repeat)?;
            traverse_actions(&operator.release)?;
        }
    };

    Ok(())
}

fn traverse_remap(keymap: &HashMap<KeyPress, Vec<KeymapAction>>) -> anyhow::Result<()> {
    for (_, actions) in keymap {
        traverse_actions(actions)?;
    }

    Ok(())
}

fn traverse_actions(actions: &Vec<KeymapAction>) -> anyhow::Result<()> {
    for action in actions {
        match action {
            KeymapAction::Remap(remap) => {
                traverse_remap(&remap.remap)?;
            }
            KeymapAction::KeyPressAndRelease(key_press) | KeymapAction::WithMark(key_press) => {
                traverse_keymap_output_keys(&vec![key_press.key])?;
            }
            KeymapAction::KeyPress(key) | KeymapAction::KeyRepeat(key) | KeymapAction::KeyRelease(key) => {
                traverse_keymap_output_keys(&vec![*key])?;
            }
            _ => {}
        }
    }

    Ok(())
}
