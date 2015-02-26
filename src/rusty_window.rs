extern crate "bootstrap-rs" as bootstrap;

use bootstrap::window::Window;
use bootstrap::gl_render;
use bootstrap::gl_render::Mesh;

fn main() {
    let instance = bootstrap::main_instance();
    let window = Window::new("Rust Window", instance);

    gl_render::init_opengl(&window);
    gl_render::create_gl_context(&window);

    loop {
        window.handle_messages();
    };
}
