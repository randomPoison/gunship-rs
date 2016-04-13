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

    /// Gets the current position of the anchor.
    pub fn position(&self) -> Point {
        self.position
    }

    /// Sets the position of the anchor.
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    /// Gets the current orientation of the anchor.
    pub fn orientation(&self) -> Quaternion {
        self.orientation
    }

    /// Sets the orientation of the anchor.
    pub fn set_orientation(&mut self, orientation: Quaternion) {
        self.orientation = orientation;
    }

    /// Gets the current scale of the anchor.
    pub fn scale(&self) -> Vector3 {
        self.scale
    }

    /// Sets the scale of the anchor.
    pub fn set_scale(&mut self, scale: Vector3) {
        self.scale = scale;
    }

    /// Attaches a mesh to the anchor, allowing the mesh to be rendered in the scene.
    pub fn attach_mesh(&mut self, gpu_mesh: GpuMesh) {
        self.meshes.insert(gpu_mesh);
    }

    /// Gets the set of meshes attached to the anchor.
    pub fn meshes(&self) -> &HashSet<GpuMesh> {
        &self.meshes
    }

    /// Removes the mesh from the anchor.
    pub fn detach_mesh(&mut self, gpu_mesh: GpuMesh) {
        self.meshes.remove(&gpu_mesh);
    }
}
