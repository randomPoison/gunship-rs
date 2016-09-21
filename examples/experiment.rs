#[macro_use]
extern crate gunship;

use gunship::async::*;
use gunship::async::engine::EngineBuilder;

fn main() {
    let mut builder = EngineBuilder::new();
    builder.max_workers(4);
    builder.build(|| {
        setup_scene();
    });

    // ENGINE HAS BEEN SHUT DOWN!
}

/// Things to do:
///
/// 1. Load and create mesh resource.
/// 2. Load and create material resource.
/// 3. Create entity in scene and assign it a mesh and material.
/// 4. Create entity in scene and assign it the camera.
fn setup_scene() {
    let (mesh, material) = await_all!(
        resource::load_mesh("examples/meshes/gun_small.dae"),
        resource::load_material("lib/polygon_rs/resources/materials/diffuse_flat.material"));

    println!("received mesh: {:?}, material: {:?}", mesh, material);

    // let mesh_transform = Transform::new();
    // let mesh_renderer = MeshRenderer::new(&mesh, &mesh_transform);
    //
    // let camera_transform = Transform::new();
    // let camera = Camera::new(&camera_transform);

    // TODO: Wait until the game is done running?
    loop {}
}
