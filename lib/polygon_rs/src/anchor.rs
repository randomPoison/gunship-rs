use math::*;
use std::collections::HashSet;
use super::GpuMesh;

#[derive(Debug)]
pub struct Anchor {
    position: Point,
    orientation: Quaternion,
    scale: Vector3,

    meshes: HashSet<GpuMesh>,
}

impl Anchor {
    /// Creates a new anchor.
    pub fn new() -> Anchor {
        Anchor {
            position: Point::origin(),
            orientation: Quaternion::identity(),
            scale: Vector3::one(),

            meshes: HashSet::new(),
        }
    }

    /// Attaches a mesh to the anchor, allowing the mesh to be rendered in the scene.
    pub fn attach_mesh(&mut self, gpu_mesh: GpuMesh) {
        self.meshes.insert(gpu_mesh);
    }

    pub fn meshes(&self) -> &HashSet<GpuMesh> {
        &self.meshes
    }

    /// Removes the mesh from the anchor.
    pub fn detach_mesh(&mut self, gpu_mesh: GpuMesh) {
        self.meshes.remove(&gpu_mesh);
    }
}
