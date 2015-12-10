use std::collections::HashSet;

use bootstrap::window::Message;
use bootstrap::window::Message::*;
pub use bootstrap::input::ScanCode;

pub const MAX_SUPPORTED_MOUSE_BUTTONS: usize = 5;

#[derive(Debug, Clone)]
pub struct Input {
    keys_pressed: HashSet<ScanCode>,
    keys_released: HashSet<ScanCode>,
    keys_down: HashSet<ScanCode>,
    mouse_pos: (i32, i32),
    mouse_delta: (i32, i32),
    mouse_down: [bool; MAX_SUPPORTED_MOUSE_BUTTONS],
    mouse_pressed: [bool; MAX_SUPPORTED_MOUSE_BUTTONS],
    mouse_released: [bool; MAX_SUPPORTED_MOUSE_BUTTONS],
    mouse_scroll: i32,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            keys_down: HashSet::new(),
            mouse_pos: (400, 400),
            mouse_delta: (0, 0),
            mouse_down: [false; MAX_SUPPORTED_MOUSE_BUTTONS],
            mouse_pressed: [false; MAX_SUPPORTED_MOUSE_BUTTONS],
            mouse_released: [false; MAX_SUPPORTED_MOUSE_BUTTONS],
            mouse_scroll: 0,
        }
    }

    pub fn clear(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta = (0, 0);
        self.mouse_pressed = [false; MAX_SUPPORTED_MOUSE_BUTTONS];
        self.mouse_released = [false; MAX_SUPPORTED_MOUSE_BUTTONS];
        self.mouse_scroll = 0;
    }

    pub fn push_input(&mut self, message: Message) {
        match message {
            KeyDown(key) => {
                if !self.key_down(key) {
                    self.keys_pressed.insert(key);
                }

                self.keys_down.insert(key);
            },
            KeyUp(key) => {
                self.keys_released.insert(key);
                self.keys_down.remove(&key);
            },
            MouseMove(x_delta, y_delta) => {
                self.mouse_delta = (x_delta, y_delta);
            },
            MousePos(x_pos, y_pos) => {
                self.mouse_pos = (x_pos, y_pos);
            },
            MouseButtonPressed(button) => {
                let index = button as usize;
                assert!(index < MAX_SUPPORTED_MOUSE_BUTTONS);

                self.mouse_down[index] = false;
                self.mouse_released[index] = true;
            },
            MouseButtonReleased(button) => {
                let index = button as usize;
                assert!(index < MAX_SUPPORTED_MOUSE_BUTTONS);

                self.mouse_pressed[index] = true ^ self.mouse_down[index];
                self.mouse_down[index] = true;
            },
            MouseWheel(scroll_amount) => {
                self.mouse_scroll = scroll_amount;
            }
            _ => panic!("Unhandled message {:?} passed to Input::push_input()", message) // TODO: Don't panic? Should be unreachable in release.
        }
    }

    pub fn key_down(&self, key: ScanCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn key_pressed(&self, key: ScanCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn key_released(&self, key: ScanCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> (i32, i32) {
        self.mouse_delta
    }

    pub fn mouse_button_down(&self, button: usize) -> bool {
        assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

        self.mouse_down[button]
    }

    pub fn mouse_button_pressed(&self, button: usize) -> bool {
        assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

        self.mouse_pressed[button]
    }

    pub fn mouse_button_released(&self, button: usize) -> bool {
        assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

        self.mouse_released[button]
    }

    pub fn mouse_scroll(&self) -> i32 {
        self.mouse_scroll
    }
}
