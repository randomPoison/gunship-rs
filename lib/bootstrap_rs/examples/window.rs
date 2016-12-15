extern crate bootstrap_rs as bootstrap;

use bootstrap::window::*;

fn main() {
    let mut window = Window::new("Bootstrap Window").unwrap();

    for message in window.message_pump() {
        println!("message: {:?}", message);
        if let Message::Close = message {
            break;
        }
    }
}
