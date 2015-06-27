#[cfg(windows)]
pub use windows::time::Timer;

#[cfg(linux)]
pub use linux::time::Timer;
