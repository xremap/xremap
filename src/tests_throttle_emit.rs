use crate::event_handler::{PRESS, RELEASE};
use crate::throttle_emit::ThrottleEmit;
use anyhow::Result;
use evdev::KeyCode;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const DELAY: Duration = Duration::from_millis(10);
/// Allow for inconsistency in test cases. Tests measure time
/// from event start to sleep_if_needed returns, but the time that
/// actually matters is from _previous_event_ start to sleep_if_needed returns.
const MINIMUM_DELAY: Duration = Duration::from_millis(9);

fn get_handler() -> ThrottleEmit {
    let handler = ThrottleEmit::new(DELAY);

    sleep(DELAY); // Ensure handler is in base state.

    handler
}

fn has_delay(handler: &mut ThrottleEmit, key: KeyCode, value: i32) -> Result<bool> {
    let time = SystemTime::now();
    handler.sleep_if_needed(key, value);
    Ok(time.elapsed()? > MINIMUM_DELAY)
}

#[test]
fn test_press_key_then_release_same_key_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    assert!(has_delay(&mut handler, KeyCode::KEY_A, RELEASE)?);

    Ok(())
}

#[test]
fn test_press_key_then_release_same_key_with_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, RELEASE)?);

    Ok(())
}

#[test]
fn test_press_key_then_release_other_key_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    assert!(!has_delay(&mut handler, KeyCode::KEY_B, RELEASE)?);

    Ok(())
}

#[test]
fn test_press_key_then_press_mod_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    assert!(has_delay(&mut handler, KeyCode::KEY_LEFTALT, PRESS)?);

    Ok(())
}

#[test]
fn test_press_key_then_press_mod_with_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyCode::KEY_LEFTALT, PRESS)?);

    Ok(())
}

#[test]
fn test_press_key_then_release_mod_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    assert!(has_delay(&mut handler, KeyCode::KEY_LEFTALT, RELEASE)?);

    Ok(())
}

#[test]
fn test_press_key_then_release_mod_with_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyCode::KEY_LEFTALT, RELEASE)?);

    Ok(())
}

#[test]
fn test_press_mod_then_press_key_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_LEFTALT, PRESS)?);

    assert!(has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);

    Ok(())
}

#[test]
fn test_press_mod_then_release_key_without_delay() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_LEFTALT, PRESS)?);

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, RELEASE)?);

    Ok(())
}

#[test]
fn test_both_press_and_release_delays_after_mod() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_LEFTCTRL, PRESS)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_V, PRESS)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_V, RELEASE)?);

    Ok(())
}

#[test]
fn test_two_different_keys_pressed_around_mod_press() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_LEFTCTRL, PRESS)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_B, PRESS)?);

    Ok(())
}

#[test]
fn test_two_different_keys_pressed_around_mod_release() -> Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyCode::KEY_A, PRESS)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_LEFTCTRL, RELEASE)?);
    assert!(has_delay(&mut handler, KeyCode::KEY_B, PRESS)?);

    Ok(())
}
