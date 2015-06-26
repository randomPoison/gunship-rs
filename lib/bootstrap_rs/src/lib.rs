extern crate gl;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::init::init;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::init::init;

pub mod window;
pub mod gl_utils;
pub mod input;
pub mod time;
