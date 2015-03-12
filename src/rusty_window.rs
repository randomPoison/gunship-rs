#![feature(core)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use bootstrap::window::Window;

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
    let window = Window::new("Rust Window", instance);

    gl_render::init(&window);
    gl_render::gl_test();

    loop {
        println!("handling messages");
        window.handle_messages();

        if main_window.close {
            break;
        }
    };
}
