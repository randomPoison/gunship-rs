#[cfg(windows)]
pub use windows::window::Window;

#[cfg(unix)]
pub use linux::window::Window;

use input::ScanCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Message signaling a mouse button has been pressed.
    ///
    /// This message is sent any time a mouse button has been pressed. The wrapped
    /// value is the index of button on the mouse: 0 is LMB, 1 is RMB, 2 is LMB, with
    /// other button assignments being driver dependent (I assume).
    MouseButtonPressed(u8),

    /// Message signaling a mouse button has been released.
    ///
    /// This message is sent any time a mouse button has been released. The wrapped
    /// value is the index of button on the mouse: 0 is LMB, 1 is RMB, 2 is LMB, with
    /// other button assignments being driver dependent (I assume).
    MouseButtonReleased(u8),

    /// Message signalling how much the mouse wheel has been scrolled.
    ///
    /// This message is sent any time the mouse wheel is scrolled. The wrapped value
    /// is the amount the mouse wheel was scrolled, though the scale of this value
    /// is platform/driver dependent (I assume).
    MouseWheel(i32),
}
