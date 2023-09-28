use clap::ValueEnum;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchTargets {
    /// add new devices automatically
    Device,
    /// reload the config automatically
    Config,
}
