extern crate bootstrap_rs as bootstrap;

use bootstrap::window::*;

fn main() {
    let mut window = Window::new("bootstrap - window example");

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }
    }
}
