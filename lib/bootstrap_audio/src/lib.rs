#[cfg(windows)]
#[path="windows.rs"]
mod audio_impl;

#[cfg(linux)]
#[path="linux.rs"]
mod audio_impl;

pub use audio_impl::{AudioSource, init};
