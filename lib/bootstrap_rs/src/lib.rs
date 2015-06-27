#![feature(str_utf16)] // TODO: Only used by windows currently, but has to be specified at root level.

extern crate gl;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::init::init;

#[cfg(linux)]
pub mod linux;

#[cfg(linux)]
pub use linux::init::init;

pub mod window;
pub mod gl_utils;
pub mod input;
pub mod time;
