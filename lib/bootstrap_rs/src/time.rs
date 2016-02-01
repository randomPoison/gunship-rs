#[cfg(target_os = "windows")]
pub use windows::time::*;

#[cfg(target_os = "linux")]
pub use linux::time::*;

#[cfg(target_os = "macos")]
pub use macos::time::*;
