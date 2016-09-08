extern crate bootstrap_rs as bootstrap;

use bootstrap::window::*;

fn main() {
    let mut window = Window::new("Bootstrap Window");

    for message in &mut window {
        println!("message: {:?}", message);
        if let Message::Close = message {
            break;
        }
    }
}
