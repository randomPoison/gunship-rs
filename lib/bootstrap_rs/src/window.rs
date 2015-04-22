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

    /// The x movement and y movement in pixels.
    MouseMove(i32, i32),

    /// The x and y coordinates in pixels.
    ///
    /// These coordinates are relative to the window, with the upper-left corner
    /// of the window being (0, 0).
    MousePos(i32, i32),
}
