use std::error::Error;

mod input;

fn main() -> Result<(), Box<dyn Error>> {
    let mut device = input::select_device();
    device.grab()?;
    device.ungrab()?;
    Ok(())
}
