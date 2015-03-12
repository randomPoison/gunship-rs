#![feature(core)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use bootstrap::window::{Window, Message};
use bootstrap::window::Message::*;

mod render;
mod gl_render;

struct MainWindow
{
    close: bool
}

fn main() {
    let mut main_window = MainWindow {
        close: false
    };

    println!("initializing bootstrap");
    let instance = bootstrap::init();

    println!("creating window");
    let mut window = Window::new("Rust Window", instance);

    gl_render::init(&window);
    gl_render::gl_test();

    loop {
        window.handle_messages();

        // handle messages
        loop {
            match window.next_message() {
                Some(message) => {
                    println!("message: {:?}", message);
                    match message {
                        Activate => (),
                        Close => main_window.close = true,
                        Destroy => (),
                        Paint => ()
                    }
                },
                None => break
            }
        }

        if main_window.close {
            break;
        }
    };
}
