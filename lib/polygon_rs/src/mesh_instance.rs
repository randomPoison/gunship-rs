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
use material::*;

/// Represents an instance of a mesh in the scene.
///
/// By default a mesh instance will not be attached to an anchor, and will not be rendered in
/// the scene until one is set with `set_anchor()` and the mesh instance is registered with
/// the renderer using `Renderer::register_mesh_instance()`.
#[derive(Debug)]
pub struct MeshInstance {
    mesh: GpuMesh,
    material: MaterialType,
    anchor: Option<AnchorId>
}

impl MeshInstance {
    /// Creates a new mesh instance sharing the specified material.
    pub fn with_shared_material(mesh: GpuMesh, material: MaterialId) -> MeshInstance {
        MeshInstance {
            mesh: mesh,
            material: MaterialType::Shared(material),
            anchor: None,
        }
    }

    /// Creates a new mesh instance with its own material.
    pub fn with_owned_material(mesh: GpuMesh, material: Material) -> MeshInstance {
        MeshInstance {
            mesh: mesh,
            material: MaterialType::Owned(material),
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

    /// Gets a reference to either the shared material ID or the owned material.
    pub fn material_type(&self) -> &MaterialType {
        &self.material
    }

    /// Gets the shared material ID if the mesh instance is using a shared material.
    pub fn shared_material(&self) -> Option<MaterialId> {
        match self.material {
            MaterialType::Shared(id) => Some(id),
            _ => None,
        }
    }

    /// Gets a reference to the material used by the mesh instance if it owns its material.
    pub fn material(&self) -> Option<&Material> {
        match self.material {
            MaterialType::Owned(ref material) => Some(material),
            _ => None,
        }
    }

    /// Gets a mutable reference to the material used by the mesh instance if it owns its material.
    pub fn material_mut(&mut self) -> Option<&mut Material> {
        match self.material {
            MaterialType::Owned(ref mut material) => Some(material),
            _ => None,
        }
    }

    /// Attaches the mesh instance to the specified anchor.
    pub fn set_anchor(&mut self, anchor_id: AnchorId) {
        self.anchor = Some(anchor_id);
    }

    /// Gets a reference to the anchor this instance is attached to.
    pub fn anchor(&self) -> Option<AnchorId> {
        self.anchor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MeshInstanceId(usize);
derive_Counter!(MeshInstanceId);
