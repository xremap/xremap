use std::error::Error;

mod input;
mod select;

fn main() -> Result<(), Box<dyn Error>> {
    let mut device = input::select_device();
    device.grab()?;
    if select::is_readable(&mut device) {
        println!("event!")
    }
    device.ungrab()?;
    Ok(())
}
