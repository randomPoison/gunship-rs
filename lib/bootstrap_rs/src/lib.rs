#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::init::init;

#[cfg(unix)]
pub mod linux;

#[cfg(unix)]
pub use linux::init::init;

pub mod window;
pub mod input;
pub mod time;
