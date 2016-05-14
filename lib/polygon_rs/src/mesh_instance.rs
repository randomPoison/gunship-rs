//! Mesh instances.
//!
//! When mesh data is sent to the GPU a `GpuMesh` is created to allow that mesh data to be
//! referenced within polygon. In order to display a mesh in the scene we use mesh instances,
//! represented by the `MeshInstance` type. Mesh instances serve two purposes:
//!
//! * Allowing meshes to be displayed numerous times in the same scene.
//! * Associating materials with meshes in the scene.

use {GpuMesh};
use anchor::AnchorId;
use material::Material;

/// Represents an instance of a mesh in the scene.
#[derive(Debug)]
pub struct MeshInstance {
    mesh: GpuMesh,
    material: Material,
    anchor: Option<AnchorId>
}

impl MeshInstance {
    /// Creates a new mesh instance for the specified mesh.
    ///
    /// By default a mesh instance will not be attached to an anchor, and will not be rendered in
    /// the scene until one is set with `set_anchor()` and the mesh instance is registered with
    /// the renderer using `Renderer::register_mesh_instance()`.
    pub fn new(mesh: GpuMesh, material: Material) -> MeshInstance {
        MeshInstance {
            mesh: mesh,
            material: material,
            anchor: None,
        }
    }

    /// Sets the mesh referenced by the mesh instance.
    pub fn set_mesh(&mut self, mesh: GpuMesh) {
        self.mesh = mesh;
    }

    /// Gets a reference to the mesh referenced by the mesh instance.
    pub fn mesh(&self) -> &GpuMesh {
        &self.mesh
    }

    /// Sets the material used by the mesh instance.
    pub fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    /// Gets a reference to the material used by the mesh instance.
    pub fn material(&self) -> &Material {
        &self.material
    }

    /// Gets a mutable reference to the material used by the mesh instance.
    pub fn material_mut(&mut self) -> &mut Material {
        &mut self.material
    }

    /// Attaches the mesh instance to the specified anchor.
    pub fn set_anchor(&mut self, anchor_id: AnchorId) {
        self.anchor = Some(anchor_id);
    }

    /// Gets a reference to the anchor this instance is attached to.
    pub fn anchor(&self) -> Option<&AnchorId> {
        self.anchor.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MeshInstanceId(usize);
derive_Counter!(MeshInstanceId);
