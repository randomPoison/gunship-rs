use std::collections::HashSet;

use bootstrap::window::Message;
use bootstrap::window::Message::*;
use bootstrap::input::ScanCode;

pub struct Input {
    pressed: HashSet<ScanCode>,
    released: HashSet<ScanCode>,
    down: HashSet<ScanCode>,
    mouse_pos: (i32, i32),
    mouse_delta: (i32, i32)
}

impl Input {
    pub fn new() -> Input {
        Input {
            pressed: HashSet::new(),
            released: HashSet::new(),
            down: HashSet::new(),
            mouse_pos: (400, 400),
            mouse_delta: (0, 0)
        }
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.mouse_delta = (0, 0);
    }

    pub fn push_input(&mut self, message: Message) {
        match message {
            KeyDown(key) => {
                self.pressed.insert(key);
                self.down.insert(key);
            },
            KeyUp(key) => {
                self.released.insert(key);
                self.down.remove(&key);
            },
            MouseMove(x_pos, y_pos) => {
                let (old_x, old_y) = self.mouse_pos;
                self.mouse_delta = (old_x - x_pos, old_y - y_pos);
                self.mouse_pos = (x_pos, y_pos);
            },
            _ => panic!("Non-input message passed to Input::push_input()")
        }
    }

    pub fn down(&self, key: ScanCode) -> bool {
        self.down.contains(&key)
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> (i32, i32) {
        self.mouse_delta
    }
}
