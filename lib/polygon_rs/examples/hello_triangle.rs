extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::math::*;
use polygon::mesh_instance::*;
use polygon::geometry::mesh::*;

static VERTEX_POSITIONS: [f32; 12] = [
    -1.0, -1.0, 0.0, 1.0,
     1.0, -1.0, 0.0, 1.0,
     0.0,  1.0, 0.0, 1.0,
];

static INDICES: [u32; 3] = [0, 1, 2];

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!").unwrap();
    let mut renderer = RendererBuilder::new(&window).build();

    // Build a triangle mesh.
    let mesh = MeshBuilder::new()
        .set_position_data(Point::slice_from_f32_slice(&VERTEX_POSITIONS))
        .set_indices(&INDICES)
        .build()
        .unwrap();

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Create an anchor and register it with the renderer.
    let anchor = Anchor::new();
    let anchor_id = renderer.register_anchor(anchor);

    // Setup the material for the mesh.
    let mut material = renderer.default_material();
    material.set_color("surface_color", Color::rgb(1.0, 0.0, 0.0));

    // Create a mesh instance, attach it to the anchor, and register it.
    let mut mesh_instance = MeshInstance::with_owned_material(gpu_mesh, material);
    mesh_instance.set_anchor(anchor_id);
    renderer.register_mesh_instance(mesh_instance);

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 10.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    // Set ambient color to pure white so we don't need to worry about lighting.
    renderer.set_ambient_light(Color::rgb(1.0, 1.0, 1.0));

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
            anchor.set_orientation(orientation + Orientation::from_eulers(0.0, 0.0, 0.0005));
        }

        // Render the mesh.
        renderer.draw();
    }
}
