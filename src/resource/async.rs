use std::path::Path;
use std::time::Instant;

/// Loads a mesh from disk.
///
/// Loads a mesh data from the specified path and performs any necessary processing to prepare it
/// to be used in rendering.
pub fn load_mesh<P: AsRef<Path>>(_path: P) -> Result<Mesh, LoadMeshError> {
    let start = Instant::now();

    // TODO: Use something like `WaitForSeconds`.
    while start.elapsed().as_secs() < 3 {}

    Ok(7)
}

// #[derive(Debug)]
pub type Mesh = usize;

#[derive(Debug)]
pub struct LoadMeshError;

pub fn load_material<P: AsRef<Path>>(_path: P) -> Result<Material, LoadMaterialError> {
    let start = Instant::now();

    // TODO: Use something like `WaitForSeconds`.
    while start.elapsed().as_secs() < 10 {}

    Err(LoadMaterialError)
}

// #[derive(Debug)]
pub type Material = f32;

#[derive(Debug)]
pub struct LoadMaterialError;
