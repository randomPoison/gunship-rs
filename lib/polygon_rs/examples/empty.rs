extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::math::*;

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!");
    let mut renderer = RendererBuilder::new().build();

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 10.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Render our empty scene.
        renderer.draw();
    }
}
