#[cfg(windows)]
pub use windows::time::Timer;

#[cfg(unix)]
pub use linux::time::Timer;
