#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

pub mod window;
pub mod input;
pub mod time;
