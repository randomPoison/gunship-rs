extern crate libc;

#[cfg(target_family = "windows")]
#[path="windows.rs"]
mod audio_impl;

pub use audio_impl::{AudioSource, init};
