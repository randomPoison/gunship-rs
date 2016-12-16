use engine::{self, EngineMessage};
use resource::{Mesh, MeshId};
use transform::Transform;
use std::marker::PhantomData;
use std::mem;

#[derive(Debug)]
pub struct MeshRenderer {
    data: *mut MeshRendererData,
    _phantom: PhantomData<MeshRendererData>,
}

impl MeshRenderer {
    pub fn new(mesh: &Mesh, transform: &Transform) -> MeshRenderer {
        let mut data = Box::new(MeshRendererData {
            mesh_id: mesh.id(),
        });

        let ptr = &mut *data as *mut _;

        engine::send_message(EngineMessage::MeshInstance(data, transform.inner()));

        MeshRenderer {
            data: ptr,
            _phantom: PhantomData,
        }
    }

    pub fn forget(self) {
        mem::forget(self);
    }
}

unsafe impl Send for MeshRenderer {}

#[derive(Debug)]
pub struct MeshRendererData {
    mesh_id: MeshId
}

impl MeshRendererData {
    pub fn mesh_id(&self) -> MeshId { self.mesh_id }
}
