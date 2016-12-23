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

    // Load mesh data from an OBJ file and send it to the GPU.
    let mesh = utils::load_mesh("resources/meshes/epps_head.obj").unwrap();
    let gpu_mesh = renderer.register_mesh(&mesh);

    // Load texture data from a BMP file and send it to the GPU.
    let texture = utils::load_texture("resources/textures/structured.bmp");
    let gpu_texture = renderer.register_texture(&texture);

    // Create an anchor for each of the meshes.
    let mut left_anchor = Anchor::new();
    left_anchor.set_position(Point::new(-1.5, 0.0, 0.0));
    let left_anchor_id = renderer.register_anchor(left_anchor);

    let mut middle_anchor = Anchor::new();
    middle_anchor.set_position(Point::new(0.0, 0.0, 0.0));
    let middle_anchor_id = renderer.register_anchor(middle_anchor);

    let mut right_anchor = Anchor::new();
    right_anchor.set_position(Point::new(1.5, 0.0, 0.0));
    let right_anchor_id = renderer.register_anchor(right_anchor);

    // Load the material for each of the meshes.
    let left_material_source =
        MaterialSource::from_file("resources/materials/diffuse_flat.material").unwrap();
    let mut left_material = renderer.build_material(left_material_source).unwrap();
    left_material.set_color("surface_color", Color::rgb(1.0, 1.0, 0.0));

    let middle_material_source =
        MaterialSource::from_file("resources/materials/diffuse_lit.material").unwrap();
    let mut middle_material = renderer.build_material(middle_material_source).unwrap();
    middle_material.set_color("surface_color", Color::rgb(0.0, 1.0, 1.0));
    middle_material.set_color("specular_color", Color::rgb(1.0, 1.0, 1.0));
    middle_material.set_f32("surface_shininess", 4.0);

    let right_material_source =
        MaterialSource::from_file("resources/materials/texture_diffuse_lit.material").unwrap();
    let mut right_material = renderer.build_material(right_material_source).unwrap();
    right_material.set_texture("surface_diffuse", gpu_texture);
    right_material.set_color("surface_color", Color::rgb(1.0, 1.0, 1.0));
    right_material.set_color("specular_color", Color::rgb(0.2, 0.2, 0.2));
    right_material.set_f32("surface_shininess", 3.0);

    // Create a mesh instance for each of the meshes, attach it to the anchor, and register it
    // with the renderer.
    let mut left_mesh_instance = MeshInstance::with_owned_material(gpu_mesh, left_material);
    left_mesh_instance.set_anchor(left_anchor_id);
    let left_instance_id = renderer.register_mesh_instance(left_mesh_instance);

    let mut middle_mesh_instance = MeshInstance::with_owned_material(gpu_mesh, middle_material);
    middle_mesh_instance.set_anchor(middle_anchor_id);
    renderer.register_mesh_instance(middle_mesh_instance);

    let mut right_mesh_instance = MeshInstance::with_owned_material(gpu_mesh, right_material);
    right_mesh_instance.set_anchor(right_anchor_id);
    renderer.register_mesh_instance(right_mesh_instance);

    // Create a camera and an anchor for it.
    let mut camera_anchor = Anchor::new();
    camera_anchor.set_position(Point::new(0.0, 0.0, 4.0));
    let camera_anchor_id = renderer.register_anchor(camera_anchor);
    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    // Create the light and an anchor for it.
    let light_anchor_id = renderer.register_anchor(Anchor::new());
    let mut light = Light::point(5.0, 1.0, Color::new(1.0, 1.0, 1.0, 1.0));
    light.set_anchor(light_anchor_id);
    renderer.register_light(light);

    let mut t: f32 = 0.0;
    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Change the surface color of the left mesh.
        {
            let color = Color::new(
                t.cos() * 0.5 + 0.5,
                t.sin() * 0.5 + 0.5,
                (t * 2.0).cos() * 0.5 + 0.5,
                1.0);

            let mesh_instance = renderer.get_mesh_instance_mut(left_instance_id).unwrap();
            mesh_instance
                .material_mut()
                .unwrap()
                .set_color("surface_color", color);
        }

        // Move the light back and forth between the middle and right mesh.
        {
            let anchor = renderer.get_anchor_mut(light_anchor_id).unwrap();
            anchor.set_position(Point::new(
                t.cos() * 2.0 + 0.75,
                0.0,
                2.0,
            ));
        }

        // Render the meshes.
        renderer.draw();

        t += 0.005;
    }
}
