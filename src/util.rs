use anyhow::{bail, Result};
use evdev::{AttributeSetRef, EvdevEnum};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[allow(dead_code)]
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

#[allow(dead_code)]
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

pub fn evdev_enums_to_string<T>(enums: &AttributeSetRef<T>) -> String
where
    T: std::fmt::Debug,
    T: EvdevEnum,
{
    enums
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Prints a table to stdout with dynamically calculated widths.
pub fn print_table(table: Vec<Vec<String>>) {
    let mut column_widths: Vec<usize> = table[0].iter().map(|_| 0).collect();
    let column_count = column_widths.len();

    // Find max column widths
    for row in &table {
        for n in 0..column_count {
            column_widths[n] = std::cmp::max(column_widths[n], row[n].chars().count());
        }
    }

    for row in table {
        for n in 0..column_count {
            let padding_size = column_widths[n] - row[n].chars().count();
            let padding = std::iter::repeat(" ").take(padding_size + 1).collect::<String>();
            print!("{}{}", row[n], padding);
        }
        println!("");
    }
}
