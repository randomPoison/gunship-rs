use window::Window;

#[derive(Debug, Copy, Clone)]
pub struct GLContext;

pub fn init(_window: &Window) {
    println!("gl::init() is not implemented on linux");
}

pub fn create_context(_window: &Window) -> GLContext {
    println!("gl::create_context() is not implemented on linux");
    GLContext
}

pub fn set_proc_loader() {
    println!("gl::set_proc_loader() is not implemented on linux");
}

pub fn swap_buffers() {
    println!("gl::swap_buffers() not implemented on linux");
}
