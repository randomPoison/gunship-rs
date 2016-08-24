use async::Future;
use std::path::Path;

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
            Err(LoadMeshError)
        }
    }

    LoadMesh
}

#[derive(Debug)]
pub struct Mesh;

#[derive(Debug)]
pub struct LoadMeshError;
