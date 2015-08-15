#[cfg(target_os = "windows")]
pub use windows::time::Timer;

#[cfg(target_os = "linux")]
pub use linux::time::Timer;
