use evdev::EventType;
use std::env;
use std::error::Error;
use std::process::exit;

mod config;
mod input;
mod output;
mod transform;

fn event_loop() -> Result<(), Box<dyn Error>> {
    let mut input_device =
        input::select_device().map_err(|e| format!("Failed to open an input device: {}", e))?;
    let mut output_device = output::build_device(&input_device)
        .map_err(|e| format!("Failed to build an output device: {}", e))?;
    input_device
        .grab()
        .map_err(|e| format!("Failed to grab an input device: {}", e))?;

    loop {
        if !input::is_readable(&mut input_device)? {
            continue;
        }

        for event in input_device.fetch_events()? {
            if event.event_type() == EventType::KEY {
                transform::on_event(event, &mut output_device)?;
            } else {
                output_device.emit(&[event])?;
            }
        }
    }
}

fn main() {
    let filename = match env::args().nth(1) {
        Some(filename) => filename,
        None => {
            println!("Usage: xremap <file>");
            exit(1);
        }
    };
    let config = match config::load_config(&filename) {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load config '{}': {}", filename, e);
            exit(1);
        }
    };
    println!("{:#?}", config);

    match event_loop() {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    }
}
