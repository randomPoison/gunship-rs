use input::ScanCode;
use platform;

/// Represents an open window on the host machine.
#[derive(Debug)]
pub struct Window(platform::window::Window);

impl Window {
    /// Creates a new window named `name`.
    pub fn new(name: &str) -> Result<Window, CreateWindowError> {
        Ok(Window(platform::window::Window::new(name)))
    }

    /// Removes and returns the next pending message from the message queue.
    ///
    /// If no messages are pending returns `None` and does not block.
    pub fn next_message(&mut self) -> Option<Message> {
        self.0.next_message()
    }

    /// Removes and returns the next pending message, blocking until one is available.
    ///
    /// If there are no pending messages in the message queue this method blocks until
    /// the next one is available. This method may still return `None` if it is not
    /// possible for the window to yield more messages (e.g. if the window is closed).
    pub fn wait_message(&mut self) -> Option<Message> {
        self.0.wait_message()
    }

    /// Gets the bounds describing the position/size of the window.
    // TODO: Return more structured, less platform-specific data.
    pub fn get_rect(&self) -> (i32, i32, i32, i32) {
        self.0.get_rect()
    }

    /// Creates a message pump for the window.
    ///
    /// A message pump allows message processing for a window to be offloaded to a worker thread
    /// without having to give away ownership of the window.
    pub fn message_pump(&mut self) -> MessagePump {
        MessagePump(self.0.inner())
    }

    /// Gets a reference to the platform-specific implementation of the window.
    pub fn platform(&self) -> &platform::window::Window {
        &self.0
    }
}

impl<'a> Iterator for &'a mut Window {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.next_message()
    }
}

#[derive(Debug)]
pub enum CreateWindowError {
}

/// An iterator that does message processing for `Window`.
pub struct MessagePump(platform::window::WindowInner);

impl MessagePump {
    /// Pumps all messages for the window, blocking until there are no more messages to be pumped.
    pub fn run(&mut self) {
        self.0.pump_forever();
    }
}

impl Iterator for MessagePump {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.0.wait_message()
    }
}

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
    // TODO: Change the origin to be the lower-left corner instead.
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
