extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!").unwrap();
    let mut renderer = RendererBuilder::new(&window).build();

    'outer: loop {
        while let Some(message) = window.next_message() {
            if let Message::Close = message {
                break 'outer;
            }
        }

        // Render our empty scene.
        renderer.draw();
    }
}
