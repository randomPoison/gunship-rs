extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::light::*;
use polygon::math::*;
use polygon::mesh_instance::*;

pub mod utils;

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!").unwrap();
    let mut renderer = RendererBuilder::new(&window).build();

    // Build a triangle mesh.
    let mesh = utils::load_mesh("resources/meshes/epps_head.obj").unwrap();

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Create an anchor and register it with the renderer.
    let mut anchor = Anchor::new();
    anchor.set_position(Point::new(0.0, 0.0, 0.0));
    let mesh_anchor_id = renderer.register_anchor(anchor);

    let mut material = renderer.default_material();
    material.set_color("surface_color", Color::rgb(1.0, 0.0, 0.0));
    material.set_color("surface_specular", Color::rgb(1.0, 1.0, 1.0));
    material.set_f32("surface_shininess", 4.0);

    // Create a mesh instance, attach it to the anchor, and register it with the renderer.
    let mut mesh_instance = MeshInstance::with_owned_material(gpu_mesh, material);
    mesh_instance.set_anchor(mesh_anchor_id);
    renderer.register_mesh_instance(mesh_instance);

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 2.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);

    // Create the light and an anchor for it.
    let light = Light::directional(Vector3::new(1.0, -1.0, -1.0), 0.25, Color::rgb(1.0, 1.0, 1.0));
    renderer.register_light(light);

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    'outer: loop {
        while let Some(message) = window.next_message() {
            if let Message::Close = message { break 'outer; }
        }

        // Render the mesh.
        renderer.draw();
    }
}
