#[cfg(target_family = "windows")]
pub use windows::window::Window;

pub trait WindowFocus {
    fn on_focus(&mut self);
}

pub trait WindowClose {
    fn on_close(&mut self);
}
