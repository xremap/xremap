use evdev::{Device, EventType};
use std::error::Error;

mod input;
mod output;
mod select;

fn event_loop(input_device: &mut Device) -> Result<(), Box<dyn Error>> {
    let mut output_device = output::build_device(input_device).unwrap();
    loop {
        if !select::is_readable(input_device) {
            continue;
        }

        for event in input_device.fetch_events().unwrap() {
            if event.event_type() == EventType::KEY {
                println!("event: {:?}", event);
            }
            output_device.emit(&[event]).unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut device = input::select_device();
    device.grab()?;
    event_loop(&mut device)?;
    device.ungrab()?;
    Ok(())
}
