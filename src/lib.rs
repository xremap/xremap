#![cfg_attr(target_os = "freebsd", allow(dead_code, unused_imports, unused_variables))]

// Fine-grained public exports
pub use crate::event::KeyValue;
pub use crate::main_impl::xremap_cli;
pub use crate::plugin::{NoopPlugin, Plugin};
pub use anyhow::Result;
pub use evdev::KeyCode;

// Exports used by integration/e2e test cases.
pub mod private {
    pub use crate::device::{select_input_devices, SEPARATOR};
    pub use crate::util::{until, until_value};
}

// Modules
#[cfg(target_os = "freebsd")]
mod platform_freebsd;
#[cfg(target_os = "linux")]
mod platform_linux;

#[cfg(test)]
mod tests;

mod action;
mod action_dispatcher;
mod bridge;
mod client;
mod command_runner;
mod config;
mod device;
mod emit_handler;
mod event;
mod event_handler;
mod main_controller;
mod main_impl;
mod operator_double_tap;
mod operator_handler;
mod operator_oneshot;
mod operator_sim;
mod operator_throttle;
mod operators;
mod plugin;
mod throttle_emit;
mod timeout_manager;
mod util;
