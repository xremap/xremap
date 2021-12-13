use std::error::Error;

mod input;
mod select;

fn main() -> Result<(), Box<dyn Error>> {
    let mut device = input::select_device();
    device.grab()?;
    for _ in 0..5 {
        if select::is_readable(&mut device) {
            for ev in device.fetch_events().unwrap() {
                println!("event: {:?}", ev);
            }
        }
    }
    device.ungrab()?;
    Ok(())
}
