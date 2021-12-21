use crate::config::Config;
use crate::input::{event_loop, select_device};
use getopts::Options;
use std::env;
use std::process::exit;

extern crate getopts;

mod client;
mod config;
mod event_handler;
mod input;
mod output;

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
    if args.opt_present("h") {
        println!("{}", &usage(&program, opts));
        return;
    }

    let filename = match &args.free.iter().map(String::as_str).collect::<Vec<&str>>()[..] {
        &[filename] => filename,
        &[..] => abort(&usage(&program, opts)),
    };
    let config = match config::load_config(&filename) {
        Ok(config) => config,
        Err(e) => abort(&format!("Failed to load config '{}': {}", filename, e)),
    };

    let devices = args.opt_strs("device");
    let input_devices = match select_device(&devices) {
        Ok(input_devices) => input_devices,
        Err(e) => abort(&format!("Failed to select devices: {}", e)),
    };

    if let Err(e) = event_loop(input_devices, &config) {
        abort(&format!("Error: {}", e));
    }
}
