#[cfg(target_os = "windows")]
pub use windows::time::*;

#[cfg(target_os = "linux")]
pub use linux::time::*;
