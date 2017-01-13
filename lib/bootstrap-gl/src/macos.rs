use bootstrap::window::Window;

#[derive(Debug, Clone, Copy)]
pub struct Context;

pub fn init(_window: &Window) {
}

pub fn create_context(_window: &Window) -> Context {
    Context
}

pub fn destroy_context(_context: Context) {
}

pub fn proc_loader(_proc_name: &str) -> Option<extern "C" fn()> {
    None
}

pub fn swap_buffers(_window: &Window) {
}
