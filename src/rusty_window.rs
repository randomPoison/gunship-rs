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
