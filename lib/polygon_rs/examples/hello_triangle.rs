extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::math::*;
use polygon::geometry::mesh::*;

static VERTEX_POSITIONS: [f32; 12] = [
    -1.0, -1.0, 0.0, 1.0,
     1.0, -1.0, 0.0, 1.0,
     0.0,  1.0, 0.0, 1.0,
];

static INDICES: [u32; 3] = [0, 1, 2];

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!");
    let mut renderer = RendererBuilder::new().build();

    // Build a triangle mesh.
    let mesh = MeshBuilder::new()
        .set_position_data(Point::slice_from_f32_slice(&VERTEX_POSITIONS))
        .set_indices(&INDICES)
        .build()
        .unwrap();

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Create an anchor, attach the mesh, and register it with the renderer.
    let mut anchor = Anchor::new();
    anchor.attach_mesh(gpu_mesh);
    let anchor_id = renderer.register_anchor(anchor);

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

        // Rotate the triangle slightly.
        {
            let anchor = renderer.get_anchor_mut(anchor_id).unwrap();
            let orientation = anchor.orientation();
            anchor.set_orientation(orientation * Quaternion::from_eulers(0.0, 0.0, 0.001));
        }

        // Render the mesh.
        renderer.draw();
    }
}