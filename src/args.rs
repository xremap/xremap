use clap::Parser;
use clap_complete::Shell;
use std::path::PathBuf;

use super::device::InputDevice;
use super::watch_targets::WatchTargets;

#[derive(Parser, Debug)]
#[clap(version)]
#[allow(non_snake_case)]
pub struct Args {
    /// Include a device name or path
    #[clap(long, use_value_delimiter = true)]
    pub device: Vec<String>,
    /// Ignore a device name or path
    #[clap(long, use_value_delimiter = true)]
    pub ignore: Vec<String>,
    /// Match mice by default
    #[clap(long)]
    pub mouse: bool,
    /// Targets to watch
    #[clap(long, value_enum, num_args = 0.., use_value_delimiter = true, require_equals = true,
           default_missing_value = "device", verbatim_doc_comment)]
    pub watch: Vec<WatchTargets>,
    /// Generate shell completions
    ///
    /// You can use them by storing in your shells completion file or by running
    /// - in bash: eval "$(xremap --completions bash)"
    /// - in fish: xremap --completions fish | source
    #[clap(long, value_enum, display_order = 100, value_name = "SHELL", verbatim_doc_comment)]
    pub completions: Option<Shell>,
    /// Config file(s)
    #[clap(required_unless_present = "completions", num_args = 1..)]
    pub configs: Vec<PathBuf>,
    #[clap(long, value_parser = validate_unique_id)]
    pub uniqueId: Option<String>,
}

fn validate_unique_id(unique_id: &str) -> Result<String, String> {
    let input_device_name_length_max = 64 - 1 - "xremap uniq=".len();

    if format!("xremap uniq={}", unique_id).len() > input_device_name_length_max {
        return Err(format!("unique-id must be {} characters or less", input_device_name_length_max));
    }

    if is_device_id_unique(unique_id).unwrap() == false {
        return Err(format!("the value {} is already in use", unique_id));
    }

    return Ok(unique_id.to_string());
}

fn is_device_id_unique(id: &str) -> Result<bool, std::io::Error> {
    let device_name = format!("xremap uniq={}", id);
    let devices: Vec<_> = InputDevice::devices()?.collect();
    let id_already_in_use = devices
        .iter()
        .any(|device| return device.device_name().contains(&device_name));
    return Ok(id_already_in_use == false);
}
