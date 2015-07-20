#![feature(str_utf16)] // TODO: Only used by windows currently, but has to be specified at root level.

extern crate gl;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::init::init;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::init::init;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::init::init;

pub mod window;
pub mod gl_utils;
pub mod input;
pub mod time;
