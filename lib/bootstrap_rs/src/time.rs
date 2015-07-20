#[cfg(windows)]
pub use windows::time::Timer;

#[cfg(target_os = "linux")]
pub use linux::time::Timer;

#[cfg(target_os = "macos")]
pub use macos::time::Timer;
