#![feature(alloc)]

extern crate "bootstrap-rs" as bootstrap;

use bootstrap::window::{Window,
                        WindowFocus};
use bootstrap::gl_render;
use bootstrap::gl_render::Mesh;

struct MainWindow
{
    value: i32
}

impl WindowFocus for MainWindow {
    fn on_focus(&self) {
        println!("main window gained focus");
    }
}

fn main() {
    let main_window = &MainWindow {
        value: -1
    };

    let instance = bootstrap::main_instance();
    let mut window = Window::new("Rust Window", instance);

    std::rc::get_mut(&mut window).unwrap().set_on_focus(main_window);

    gl_render::init_opengl(&window);
    gl_render::create_gl_context(&window);

    loop {
        window.handle_messages();
    };
}
