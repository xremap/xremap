use evdev::{Device, EventType};
use std::error::Error;
use std::env;
use std::process::exit;

mod input;
mod output;
mod select;
mod transform;
mod config;

fn event_loop(input_device: &mut Device) -> Result<(), Box<dyn Error>> {
    let mut output_device = output::build_device(input_device).unwrap();
    loop {
        if !select::is_readable(input_device) {
            continue;
        }

        for event in input_device.fetch_events().unwrap() {
            if event.event_type() == EventType::KEY {
                transform::on_event(event, &mut output_device);
            } else {
                output_device.emit(&[event]).unwrap();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = match env::args().nth(1) {
        Some(filename) => filename,
        None => {
            println!("Usage: xremap <file>");
            exit(1);
        },
    };
    let config = match config::load_config(&filename) {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load config '{}': {}", filename, e);
            exit(1);
        },
    };
    println!("{:?}", config);

    let mut device = input::select_device();
    device.grab()?;
    event_loop(&mut device)?;
    device.ungrab()?;
    Ok(())
}
