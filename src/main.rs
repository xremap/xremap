use evdev::EventType;
use std::env;
use getopts::Options;
use std::error::Error;
use std::process::exit;
use crate::config::Config;

extern crate getopts;

mod config;
mod input;
mod output;
mod transform;

fn event_loop(_config: &Config) -> Result<(), Box<dyn Error>> {
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

fn usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} CONFIG [options]", program);
    opts.usage(&brief)
}

fn abort(message: &str) -> ! {
    println!("{}", message);
    exit(1);
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let program = argv[0].clone();

    let mut opts = Options::new();
    opts.optmulti("", "device", "device name or path", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let args = match opts.parse(&argv[1..]) {
        Ok(args) => args,
        Err(e) => abort(&e.to_string()),
    };

    let filename = match &args.free.iter().map(String::as_str).collect::<Vec<&str>>()[..] {
        &[filename] => filename,
        &[..] => abort(&usage(&program, opts)),
    };
    let config = match config::load_config(&filename) {
        Ok(config) => config,
        Err(e) => abort(&format!("Failed to load config '{}': {}", filename, e)),
    };

    if let Err(e) = event_loop(&config) {
        abort(&format!("Error: {}", e));
    }
}
