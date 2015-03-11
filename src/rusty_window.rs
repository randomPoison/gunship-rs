#![feature(alloc)]

extern crate "bootstrap-rs" as bootstrap;

use bootstrap::window::{Window,
                        WindowFocus, WindowClose};
use bootstrap::gl_render;
use bootstrap::gl_render::Mesh;

struct MainWindow
{
    close: bool
}

fn main() {
    let mut main_window = MainWindow {
        close: false
    };

    println!("initializing bootstrap");
    let instance = bootstrap::main_instance();

    println!("creating window");
    let window = Window::new("Rust Window", instance);

    //window.set_on_focus(&mut main_window);
    // window.set_on_close(&mut main_window);
    //
    // gl_render::init_opengl(&window);
    // gl_render::create_gl_context(&window);
    //
    loop {
        println!("handling messages");
        window.handle_messages();

        if main_window.close {
            break;
        }
    };
}
