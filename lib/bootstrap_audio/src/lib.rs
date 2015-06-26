#[cfg(target_family = "windows")]
#[path="windows.rs"]
mod audio_impl;

#[cfg(target_os = "linux")]
#[path="linux.rs"]
mod audio_impl;

pub use audio_impl::{AudioSource, init};
