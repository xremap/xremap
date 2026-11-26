#![cfg(feature = "device-test")]

use crate::common::key_press;
use crate::common::xremap_controller::XremapController;
use evdev::KeyCode;
use indoc::indoc;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

mod common;

const ABOVE_MAX_CONNECTIONS: u8 = 100; // Max is 64

#[test]
pub fn e2e_ipc_sets_matching_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);

    // Ipc client changed mode, so now exit is possible.
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_serial_connections() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    // No problem as long they are closed serially.
    for _ in 0..ABOVE_MAX_CONNECTIONS {
        assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);
    }

    // Now also change mode
    for i in 0..ABOVE_MAX_CONNECTIONS {
        let mode = format!("new_mode_{i}");
        assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", &mode])?);
        assert_eq!(format!("{mode}\n"), ctrl.ipc_client(vec!["get-mode"])?);
    }

    // Check key handling still works
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_max_connections() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    // Expect all requests to succeed until they start all to fail.
    let mut max_reached = false;
    let mut streams = vec![];
    for _ in 0..ABOVE_MAX_CONNECTIONS {
        let reader = UnixStream::connect(ctrl.get_ipc_path()).and_then(|mut stream| {
            stream.write_all(b"get-mode\n")?;
            let mut reader = BufReader::new(stream);
            reader.read_line(&mut String::new())?;
            Ok(reader)
        });
        match reader {
            Ok(_) => assert!(!max_reached),
            Err(_) => max_reached = true,
        };
        // Keep connections open. Alternative could be `--keep-alive`
        // argument to `xremap-cli get-mode`
        streams.push(reader);
    }

    assert!(max_reached);

    // Free up connections to xremap, so functionality is restored
    drop(streams);
    assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);

    // Check key handling still works
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_key_injection_is_disabled_by_default() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    // Injection requires `ipc.inject: true`; the daemon must stay up.
    assert_eq!("err injection disabled\n", ctrl.ipc_client_failure(vec!["key", "f24"])?);
    assert_eq!("err injection disabled\n", ctrl.ipc_client_failure(vec!["keydown", "shift"])?);
    assert_eq!("err injection disabled\n", ctrl.ipc_client_failure(vec!["sleep", "1"])?);
    assert_eq!("pong\n", ctrl.ipc_client(vec!["ping"])?);
    Ok(())
}

#[test]
pub fn e2e_ipc_injected_keys_enter_the_keymap() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .ipc_inject(true)
        .config_with_ipc(&indoc! {"
            keymap:
                - remap:
                    f24: { action: exit }
            ",
        })?
        .build()?;

    // `keydown`/`keyup` are accepted; the injected key then flows through the
    // keymap (scope `remap`) and triggers the configured exit. The raw scope
    // bypasses the pipeline, so the mapping would not match.
    assert_eq!("ok\n", ctrl.ipc_client(vec!["keydown", "--scope", "remap", "f24"])?);
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_sleep_pauses_but_keeps_the_daemon_responsive() -> anyhow::Result<()> {
    let ctrl = XremapController::builder()
        .ipc_inject(true)
        .config_with_ipc(&indoc! {"
            keymap:
                - remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    assert_eq!("ok\n", ctrl.ipc_client(vec!["sleep", "1"])?);
    assert_eq!("pong\n", ctrl.ipc_client(vec!["ping"])?);
    Ok(())
}
