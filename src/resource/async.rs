use async::Future;
use std::path::Path;
use std::time::Instant;

/// Loads a mesh from disk.
///
/// Loads a mesh data from the specified path and performs any necessary processing to prepare it
/// to be used in rendering.
pub fn load_mesh<P: AsRef<Path>>(_path: P) -> impl Future<Item = Mesh, Error = LoadMeshError> {
    struct LoadMesh;

    impl Future for LoadMesh {
        type Item = Mesh;
        type Error = LoadMeshError;

        fn run(&mut self) -> Result<Mesh, LoadMeshError> {
            let start = Instant::now();

            // Spin-lock until time is up.
            while start.elapsed().as_secs() < 3 { }

            Ok(7)
        }
    }

    LoadMesh
}

// #[derive(Debug)]
pub type Mesh = usize;

#[derive(Debug)]
pub struct LoadMeshError;

pub fn load_material<P: AsRef<Path>>(_path: P) -> impl Future<Item = Material, Error = LoadMaterialError> {
    struct LoadMaterial;

    impl Future for LoadMaterial {
        type Item = Material;
        type Error = LoadMaterialError;

        fn run(&mut self) -> Result<Material, LoadMaterialError> {
            let start = Instant::now();

            // Spin-lock until time is up.
            while start.elapsed().as_secs() < 10 { }

            Err(LoadMaterialError)
        }
    }

    LoadMaterial
}

// #[derive(Debug)]
pub type Material = f32;

#[derive(Debug)]
pub struct LoadMaterialError;
