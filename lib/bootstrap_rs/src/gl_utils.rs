#[cfg(windows)]
pub use windows::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};

#[cfg(linux)]
pub use linux::gl::{
    GLContext,
    init, create_context, swap_buffers, set_proc_loader
};
