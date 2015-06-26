#[cfg(target_os = "windows")]
pub use windows::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};

#[cfg(target_os = "linux")]
pub use linux::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};
