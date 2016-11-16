#[macro_use]
extern crate gunship;

use gunship::async::*;
use gunship::async::camera::Camera;
use gunship::async::engine::EngineBuilder;
use gunship::async::mesh_renderer::MeshRenderer;
use gunship::async::transform::Transform;
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

    // // EXAMPLE: `Async<T>` lifetime parameter.
    // let async_mesh = {
    //     let path = String::from("lib/polygon_rs/resources/meshes/epps_head.obj");
    //     resource::load_mesh(&*path)
    // };

    // Await the operations, suspending this fiber until they complete.
    let mesh = async_mesh.await().unwrap();
    let _material = async_material.await().unwrap();

    let mesh_transform = Transform::new();
    let _mesh_renderer = MeshRenderer::new(&mesh, &mesh_transform);

    let camera_transform = Transform::new();
    camera_transform.set_position(Point::new(0.0, 0.0, 10.0));
    let camera = Camera::new(&camera_transform);

    let mut time: f32 = 0.0;
    engine::run_each_frame(move || {
        time += 1.0 / 60.0 * 2.0 * PI / 5.0;
        let new_pos = Point::new(
            time.cos() * 3.0,
            time.sin() * 3.0,
            0.0,
        );
        mesh_transform.set_position(new_pos);
    });

    engine::run_each_frame(move || {
        time += 1.0 / 60.0 * 2.0 * PI;
        let new_pos = Point::new(
            0.0,
            0.0,
            10.0 + time.cos() * 2.0,
        );
        camera_transform.set_position(new_pos);
    });
}
