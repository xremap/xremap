#![cfg_attr(target_os = "freebsd", allow(dead_code, unused_imports, unused_variables))]

// Fine-grained public exports
pub use crate::event::KeyValue;
pub use crate::main_impl::xremap_cli;
pub use evdev::KeyCode;

// Exports used by integration/e2e test cases.
pub mod device;
pub mod util;

// Modules
#[cfg(target_os = "freebsd")]
mod platform_freebsd;
#[cfg(target_os = "linux")]
mod platform_linux;

mod action;
mod action_dispatcher;
mod bridge;
mod client;
mod command_runner;
mod config;
mod emit_handler;
mod event;
mod event_handler;
mod main_controller;
mod main_impl;
mod operator_double_tap;
mod operator_handler;
mod operator_sim;
mod operators;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_any_key;
#[cfg(test)]
mod tests_disguised_events_in;
#[cfg(test)]
mod tests_escape_next_key;
#[cfg(test)]
mod tests_extra_modifiers;
#[cfg(test)]
mod tests_keymap_mark;
#[cfg(test)]
mod tests_keymap_mode;
#[cfg(test)]
mod tests_keymap_modifier_triggers;
#[cfg(test)]
mod tests_modmap_keys;
#[cfg(test)]
mod tests_modmap_mul_purpose;
#[cfg(test)]
mod tests_modmap_mul_purpose_tap_preferred;
#[cfg(test)]
mod tests_modmap_press_release_key;
#[cfg(test)]
mod tests_nested_remap;
#[cfg(test)]
mod tests_operator_double_tap;
#[cfg(test)]
mod tests_operator_handler;
#[cfg(test)]
mod tests_operator_sim;
#[cfg(test)]
mod tests_throttle_emit;
#[cfg(test)]
mod tests_virtual_modifier;
mod throttle_emit;
mod timeout_manager;
