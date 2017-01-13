use std::collections::HashSet;

use bootstrap;
use bootstrap::window::Message;
use bootstrap::window::Message::*;
use engine;

pub use bootstrap::input::ScanCode;

pub const MAX_SUPPORTED_MOUSE_BUTTONS: usize = 5;

pub fn set_cursor(visible: bool) {
    bootstrap::input::set_cursor_visibility(visible);
}

pub fn set_capture(capture: bool) {
    if capture {
        let (top, left, bottom, right) = engine::window(|window| window.get_rect());
        bootstrap::input::set_cursor_bounds(top, left, bottom, right);
    } else {
        bootstrap::input::clear_cursor_bounds();
    }
}

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
            mouse_pos: (400, 400), // TODO: What's up with this hard-coded garbage???
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

    fn key_down(&self, key: ScanCode) -> bool {
        self.keys_down.contains(&key)
    }
}

pub fn key_down(key: ScanCode) -> bool {
    engine::input(|input| input.keys_down.contains(&key))
}

pub fn key_pressed(key: ScanCode) -> bool {
    engine::input(|input| input.keys_pressed.contains(&key))
}

pub fn key_released(key: ScanCode) -> bool {
    engine::input(|input| input.keys_released.contains(&key))
}

pub fn mouse_pos() -> (i32, i32) {
    engine::input(|input| input.mouse_pos)
}

pub fn mouse_delta() -> (i32, i32) {
    engine::input(|input| input.mouse_delta)
}

pub fn mouse_button_down(button: usize) -> bool {
    assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

    engine::input(|input| input.mouse_down[button])
}

pub fn mouse_button_pressed(button: usize) -> bool {
    assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

    engine::input(|input| input.mouse_pressed[button])
}

pub fn mouse_button_released(button: usize) -> bool {
    assert!(button < MAX_SUPPORTED_MOUSE_BUTTONS);

    engine::input(|input| input.mouse_released[button])
}

pub fn mouse_scroll() -> i32 {
    engine::input(|input| input.mouse_scroll)
}
