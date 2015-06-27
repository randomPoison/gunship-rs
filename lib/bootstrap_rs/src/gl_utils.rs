#[cfg(windows)]
pub use windows::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};

#[cfg(unix)]
pub use linux::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};
