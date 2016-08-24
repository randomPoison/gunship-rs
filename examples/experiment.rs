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
    // Load mesh resource.
    let mesh_future = resource::async::load_mesh("examples/meshes/cube.dae");
    println!("awaiting mesh resource");
    let mesh_result = await!(mesh_future);
    println!("mesh resource returned: {:?}", mesh_result);
    // let _mesh = mesh_result.unwrap();
}
