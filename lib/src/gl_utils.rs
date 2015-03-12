#[cfg(target_family = "windows")]
pub use windows::gl::{init, create_context, swap_buffers};
