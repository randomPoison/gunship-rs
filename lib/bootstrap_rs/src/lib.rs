// Almost certainly going to be stabilized as-is, unlikely to break anything.
#![feature(const_fn)]

// The scheduler puts a `Condvar` and `Mutex` into some statics.
#![feature(drop_types_in_const)]

extern crate cell_extras;

// This `extern_crate` should be within the macos platform module, but `macro_use` only works at
// the root of the crate.
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
