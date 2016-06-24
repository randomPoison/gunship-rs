extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::light::*;
use polygon::math::*;
use polygon::mesh_instance::*;

mod utils;

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!");
    let mut renderer = RendererBuilder::new().build();

    // Build a triangle mesh.
    let mesh = utils::load_mesh("resources/meshes/epps_head.obj").unwrap();

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Create an anchor and register it with the renderer.
    let mut anchor = Anchor::new();
    anchor.set_position(Point::new(0.0, 0.0, 0.0));
    let mesh_anchor_id = renderer.register_anchor(anchor);

    // Create a mesh instance, attach it to the anchor, and register it with the renderer.
    let mut mesh_instance = MeshInstance::new(gpu_mesh, renderer.default_material());
    mesh_instance.set_anchor(mesh_anchor_id);
    mesh_instance.material_mut().set_f32("surface_shininess", 5.0);
    let instance_id = renderer.register_mesh_instance(mesh_instance);

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 2.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);

    // Create the light and an anchor for it.
    let light_anchor_id = renderer.register_anchor(Anchor::new());
    let mut light = Light::point(5.0, 3.0, Color::new(1.0, 1.0, 1.0, 1.0));
    light.set_anchor(light_anchor_id);
    renderer.register_light(light);

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    const LIGHT_RADIUS: f32 = 5.0;

    let mut t: f32 = 0.0;
    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Orbit the light around the mesh.
        {
            let anchor = renderer.get_anchor_mut(light_anchor_id).unwrap();
            anchor.set_position(Point::new(
                t.cos() * LIGHT_RADIUS,
                t.sin() * LIGHT_RADIUS,
                2.0,
            ));
        }

        // Change the surface color.
        {
            let color = Color::new(
                t.cos() * 0.5 + 0.5,
                t.sin() * 0.5 + 0.5,
                (t * 2.0).cos() * 0.5 + 0.5,
                1.0);

            let mesh_instance = renderer.get_mesh_instance_mut(instance_id).unwrap();
            mesh_instance
                .material_mut()
                .set_color("surface_color", color);
        }

        // Render the mesh.
        renderer.draw();

        t += 0.0005;
    }
}
