#[macro_use]
extern crate gunship;

use gunship::*;
use gunship::camera::Camera;
use gunship::engine::EngineBuilder;
use gunship::light::DirectionalLight;
use gunship::mesh_renderer::MeshRenderer;
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
    let mesh = resource::load_mesh("lib/polygon_rs/resources/meshes/epps_head.obj").await().unwrap();

    DirectionalLight::new(Vector3::new(1.0, -1.0, -1.0), Color::rgb(1.0, 1.0, 1.0), 0.25).forget();

    let mut camera_transform = Transform::new();
    camera_transform.set_position(Point::new(0.0, 0.0, 35.0));
    Camera::new(&camera_transform).forget(); // TODO: Don't drop the camera, it needs to stay in scope.
    camera_transform.forget();

    // -10 --- 10
    //   X X X X -10
    //   X X X X  |
    //   X X X X  |
    //   X X X X  10
    const NUM_ROWS: usize = 4;
    const SQUARE_SIZE: f32 = 20.0;

    fn coord(pos: usize) -> f32 {
        -(SQUARE_SIZE / 2.0) + (SQUARE_SIZE / NUM_ROWS as f32) * pos as f32
    }

    for row in 0..NUM_ROWS {
        for col in 0..NUM_ROWS {
            let mut mesh_transform = Transform::new();
            mesh_transform.set_position(Point::new(
                coord(col),
                coord(row),
                0.0,
            ));
            MeshRenderer::new(&mesh, &mesh_transform).forget();
            mesh_transform.forget();
        }
    }
}
