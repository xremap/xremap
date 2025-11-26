use crate::event::{KeyEvent, KeyValue};
use crate::throttle_emit::ThrottleEmit;
use evdev::KeyCode;
use evdev::KeyCode as Key;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const DELAY: Duration = Duration::from_millis(10);

fn get_handler() -> ThrottleEmit {
    let handler = ThrottleEmit::new(DELAY);

    sleep(DELAY); // Ensure handler is in base state.

    handler
}

fn has_delay(handler: &mut ThrottleEmit, key_event: KeyEvent) -> anyhow::Result<bool> {
    let time = SystemTime::now();
    handler.sleep_if_needed(KeyCode(key_event.code()), key_event.value());
    Ok(time.elapsed()? > DELAY)
}

#[test]
fn test_press_key_then_release_same_key_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    assert!(has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Release))?);

    Ok(())
}

#[test]
fn test_press_key_then_release_same_key_with_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Release))?);

    Ok(())
}

#[test]
fn test_press_key_then_release_other_key_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_B, KeyValue::Release))?);

    Ok(())
}

#[test]
fn test_press_key_then_press_mod_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    assert!(has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))?);

    Ok(())
}

#[test]
fn test_press_key_then_press_mod_with_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))?);

    Ok(())
}

#[test]
fn test_press_key_then_release_mod_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    assert!(has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release))?);

    Ok(())
}

#[test]
fn test_press_key_then_release_mod_with_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    sleep(DELAY); // So no delay is needed

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release))?);

    Ok(())
}

#[test]
fn test_press_mod_then_press_key_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))?);

    assert!(has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Press))?);

    Ok(())
}

#[test]
fn test_press_mod_then_release_key_without_delay() -> anyhow::Result<()> {
    let mut handler = get_handler();

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press))?);

    assert!(!has_delay(&mut handler, KeyEvent::new(Key::KEY_A, KeyValue::Release))?);

    Ok(())
}
