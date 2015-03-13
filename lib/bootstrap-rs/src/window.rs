#[cfg(target_family = "windows")]
pub use windows::window::Window;

#[derive(Debug)]
pub enum Message {
    Activate,
    Close,
    Destroy,
    Paint
}
