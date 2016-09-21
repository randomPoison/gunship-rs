#![feature(trace_macros)]
#![cfg_attr(target_os = "windows", feature(str_utf16))]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "windows")]
#[path="windows\\mod.rs"]
pub mod platform;

#[cfg(target_os="linux")]
#[path="linux/mod.rs"]
pub mod platform;

#[cfg(target_os = "macos")]
#[path="macos/mod.rs"]
pub mod platform;

pub mod window;
pub mod input;
pub mod time;
