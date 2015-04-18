#[cfg(target_family = "windows")]
pub use windows::window::Window;
pub use input::ScanCode;

#[derive(Debug)]
pub enum Message {
    Activate,
    Close,
    Destroy,
    Paint,
    KeyUp(ScanCode),
    KeyDown(ScanCode),
    MouseMove(i32, i32)
}
