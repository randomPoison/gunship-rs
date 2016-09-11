#[cfg(windows)]
#[path="windows\\mod.rs"]
pub mod platform;

#[cfg(target_os="linux")]
#[path="linux/mod.rs"]
pub mod platform;

pub mod window;
pub mod input;
pub mod time;
