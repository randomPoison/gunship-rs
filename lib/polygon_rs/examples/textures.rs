extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::light::*;
use polygon::material::*;
use polygon::math::*;
use polygon::mesh_instance::*;

mod utils;

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!").unwrap();
    let mut renderer = RendererBuilder::new(&window).build();

    // Build a triangle mesh.
    let mesh = utils::load_mesh("resources/meshes/epps_head.obj").unwrap();
    let texture = utils::load_texture("resources/textures/structured.bmp");

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Send the texture to the GPU.
    let gpu_texture = renderer.register_texture(&texture);

    // Create an anchor and register it with the renderer.
    let mut anchor = Anchor::new();
    anchor.set_position(Point::new(0.0, 0.0, 0.0));
    let mesh_anchor_id = renderer.register_anchor(anchor);

    let material_source =
        MaterialSource::from_file("resources/materials/texture_diffuse_lit.material").unwrap();
    let mut material = renderer.build_material(material_source).unwrap();
    material.set_color("surface_color", Color::rgb(1.0, 1.0, 1.0));
    material.set_f32("surface_shininess", 4.0);
    material.set_texture("surface_diffuse", gpu_texture);

    // Create a mesh instance, attach it to the anchor, and register it with the renderer.
    let mut mesh_instance = MeshInstance::with_owned_material(gpu_mesh, material);
    mesh_instance.set_anchor(mesh_anchor_id);
    renderer.register_mesh_instance(mesh_instance);

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 2.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);

    // Create the light and an anchor for it.
    let light_anchor_id = renderer.register_anchor(Anchor::new());
    let mut light = Light::point(LIGHT_RADIUS, 1.0, Color::new(1.0, 1.0, 1.0, 1.0));
    light.set_anchor(light_anchor_id);
    renderer.register_light(light);

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    const LIGHT_RADIUS: f32 = 2.0;

    let mut t: f32 = 0.0;
    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Rotate the mesh slightly.
        {
            let anchor = renderer.get_anchor_mut(mesh_anchor_id).unwrap();
            anchor.set_orientation(Orientation::from_eulers(0.0, 2.0, 0.0) * (t / 2.0));
        }

        // Orbit the light around the mesh.
        {
            let anchor = renderer.get_anchor_mut(light_anchor_id).unwrap();
            anchor.set_position(Point::new(
                t.cos() * LIGHT_RADIUS * 0.5,
                t.sin() * LIGHT_RADIUS * 0.5,
                0.75,
            ));
        }

        // Render the mesh.
        renderer.draw();

        t += 0.0005;
    }
}
