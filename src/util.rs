use anyhow::bail;
use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

pub fn until<F: FnMut() -> bool>(mut callback: F, timeout: Duration, message: impl Into<String>) -> Result<()> {
    let start = Instant::now();

    loop {
        if callback() {
            return Ok(());
        };

        sleep(Duration::from_millis(10));

        // Check time
        if start.elapsed() > timeout {
            bail!(message.into());
        };
    }
}

pub fn until_value<'a, T, F: FnMut() -> Option<T>>(
    mut callback: F,
    timeout: Duration,
    message: impl Into<String>,
) -> Result<T> {
    let start = Instant::now();

    loop {
        match callback() {
            None => {
                sleep(Duration::from_millis(10));

                // Check time
                if start.elapsed() > timeout {
                    bail!(message.into());
                };
            }
            Some(val) => {
                return Ok(val);
            }
        }
    }
}
