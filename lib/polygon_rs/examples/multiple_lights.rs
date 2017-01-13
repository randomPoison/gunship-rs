extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::*;
use polygon::anchor::*;
use polygon::camera::*;
use polygon::light::*;
use polygon::math::*;
use polygon::material::*;
use polygon::mesh_instance::*;

pub mod utils;

struct OrbitingLight {
    anchor_id: AnchorId,
    time_offset: f32,
    scale: Vector3,
    radius: Vector3,
}

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

    let material_source = MaterialSource::from_file("resources/materials/diffuse_lit.material").unwrap();
    let mut material = renderer.build_material(material_source).unwrap();
    material.set_color("surface_color", Color::rgb(1.0, 1.0, 1.0));
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

    let mut point_lights = Vec::new();

    // Create the point lights.
    {
        let light_anchor_id = renderer.register_anchor(Anchor::new());
        let mut light = Light::point(2.0, 1.0, Color::rgb(1.0, 0.0, 0.0));
        light.set_anchor(light_anchor_id);
        renderer.register_light(light);

        point_lights.push(OrbitingLight {
            anchor_id: light_anchor_id,
            time_offset: 2.3,
            scale: Vector3::new(1.0, 1.0, 1.0),
            radius: Vector3::new(1.0, 1.0, 1.0),
        });
    }

    {
        let light_anchor_id = renderer.register_anchor(Anchor::new());
        let mut light = Light::point(2.0, 1.0, Color::rgb(0.0, 1.0, 0.0));
        light.set_anchor(light_anchor_id);
        renderer.register_light(light);

        point_lights.push(OrbitingLight {
            anchor_id: light_anchor_id,
            time_offset: 0.0,
            scale: Vector3::new(2.0, 0.6, 1.0),
            radius: Vector3::new(1.0, 1.0, 1.0),
        });
    }

    {
        let light_anchor_id = renderer.register_anchor(Anchor::new());
        let mut light = Light::point(2.0, 1.0, Color::rgb(0.0, 0.0, 1.0));
        light.set_anchor(light_anchor_id);
        renderer.register_light(light);

        point_lights.push(OrbitingLight {
            anchor_id: light_anchor_id,
            time_offset: 1.7,
            scale: Vector3::new(0.7, 1.7, 0.1),
            radius: Vector3::new(1.0, 1.0, 1.0),
        });
    }

    let mut camera = Camera::default();
    camera.set_anchor(camera_anchor_id);
    renderer.register_camera(camera);

    let mut t: f32 = 0.0;
    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Orbit the light around the mesh.
        for orbit in &point_lights {
            let anchor = renderer.get_anchor_mut(orbit.anchor_id).unwrap();
            let position = Point::new(
                ((t + orbit.time_offset) * orbit.scale.x).sin() * orbit.radius.x,
                ((t + orbit.time_offset) * orbit.scale.y).cos() * orbit.radius.y,
                ((t + orbit.time_offset) * orbit.scale.z).sin() * orbit.radius.z,
            );
            anchor.set_position(position);
        }

        // Render the mesh.
        renderer.draw();

        t += 0.0015;
    }
}
