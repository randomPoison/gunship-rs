#[macro_use]
extern crate gunship;

use gunship::*;

fn main() {
    let mut builder = EngineBuilder::new();
    builder.max_workers(4);
    builder.build();

    setup_scene();
}

/// Things to do:
///
/// 1. Load and create mesh resource.
/// 2. Load and create material resource.
/// 3. Create entity in scene and assign it a mesh and material.
/// 4. Create entity in scene and assign it the camera.
fn setup_scene() {
    let (mesh, material) = await_all!(
        resource::async::load_mesh("examples/meshes/cube.dae"),
        resource::async::load_material("lib/polygon_rs/resources/materials/diffuse_flat.material"));

    println!("mesh: {:?}", mesh);
    println!("material: {:?}", material);
}
