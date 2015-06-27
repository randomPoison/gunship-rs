#[cfg(windows)]
#[path="windows.rs"]
mod audio_impl;

#[cfg(unix)]
#[path="linux.rs"]
mod audio_impl;

pub use audio_impl::{AudioSource, init};
