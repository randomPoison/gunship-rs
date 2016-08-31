#[macro_use]
extern crate gunship;

use gunship::*;

fn main() {
    EngineBuilder::new().build();
    setup_scene();
}

/// Things to do:
///
/// 1. Load and create mesh resource.
/// 2. Load and create material resource.
/// 3. Create entity in scene and assign it a mesh and material.
/// 4. Create entity in scene and assign it the camera.
fn setup_scene() {
    let mesh_future = resource::async::load_mesh("examples/meshes/cube.dae");
    let material_future = resource::async::load_material("lib/polygon_rs/resources/materials/diffuse_flat.material");

    let result = await_all!(mesh_future, material_future);
    println!("{:?}", result);
}
