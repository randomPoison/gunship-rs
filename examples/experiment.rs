#[macro_use]
extern crate gunship;

use gunship::*;
use gunship::camera::Camera;
use gunship::engine::EngineBuilder;
use gunship::light::DirectionalLight;
use gunship::mesh_renderer::MeshRenderer;
use gunship::stopwatch::Stopwatch;
use gunship::transform::Transform;
use gunship::math::*;

fn main() {
    let mut builder = EngineBuilder::new();
    builder.max_workers(8);
    builder.build(|| {
        setup_scene();
    });

    // ENGINE HAS BEEN SHUT DOWN!
}

/// Things to do:
///
/// 1. Load and create mesh resource.
/// 2. Load and create material resource.
/// 3. Create transform in scene and assign it a mesh and material.
/// 4. Create transform in scene and assign it the camera.
fn setup_scene() {
    // Start both async operations but don't await either, allowing both to run concurrently.
    let async_mesh = resource::load_mesh("lib/polygon_rs/resources/meshes/epps_head.obj");
    let async_material = resource::load_material("lib/polygon_rs/resources/materials/diffuse_flat.material");

    // Await the operations, suspending this fiber until they complete.
    let mesh = async_mesh.await().unwrap();
    let _material = async_material.await().unwrap();

    let mut mesh_transform = Transform::new();
    let _mesh_renderer = MeshRenderer::new(&mesh, &mesh_transform);

    let mut camera_transform = Transform::new();
    camera_transform.set_position(Point::new(0.0, 0.0, 10.0));
    let camera = Camera::new(&camera_transform); // TODO: Don't drop the camera, it needs to stay in scope.

    let light = DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0), Color::rgb(1.0, 1.0, 1.0), 0.25);

    let mut time: f32 = 0.0;
    engine::run_each_frame(move || {
        let _s = Stopwatch::new("Move mesh");

        time += time::delta_f32() * TAU / 10.0;
        let new_pos = Point::new(
            time.cos() * 3.0,
            time.sin() * 3.0,
            0.0,
        );
        mesh_transform.set_position(new_pos);
    });

    engine::run_each_frame(move || {
        let _s = Stopwatch::new("Move camera");

        time += time::delta_f32() * TAU / 3.0;
        let new_pos = Point::new(
            0.0,
            0.0,
            10.0 + time.cos() * 2.0,
        );
        camera_transform.set_position(new_pos);
    });
}
