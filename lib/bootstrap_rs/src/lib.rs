// Almost certainly going to be stabilized as-is, unlikely to break anything.
#![feature(const_fn)]

// The scheduler puts a `Condvar` and `Mutex` into some statics.
#![feature(drop_types_in_const)]

// TODO: Get rid of this nonsense. There are stable ways of handling utf16.
#![cfg_attr(target_os = "windows", feature(str_utf16))]

extern crate cell_extras;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "windows")]
#[path="windows/mod.rs"]
pub mod platform;

#[cfg(target_os = "linux")]
#[path="linux/mod.rs"]
pub mod platform;

#[cfg(target_os = "macos")]
#[path="macos/mod.rs"]
pub mod platform;

pub mod window;
pub mod input;
pub mod time;
