#[cfg(target_family = "windows")]
pub use windows::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};
