use ecs::{Entity, ComponentManager};
use polygon::gl_render::{GLMeshData, ShaderProgram};
use resource::ResourceManager;
use std::rc::Rc;
use super::struct_component_manager::*;


#[derive(Debug, Clone)]
pub struct Mesh {
    pub gl_mesh: GLMeshData,
    pub shader: ShaderProgram,
}

pub struct MeshManager {
    inner: StructComponentManager<Mesh>,
    resource_manager: Rc<ResourceManager>,
}

impl MeshManager {
    pub fn new(resource_manager: Rc<ResourceManager>) -> MeshManager {
        MeshManager {
            inner: StructComponentManager::new(),
            resource_manager: resource_manager,
        }
    }

    pub fn clone(&self, resource_manager: Rc<ResourceManager>) -> MeshManager {
        MeshManager {
            inner: self.inner.clone(),
            resource_manager: resource_manager,
        }
    }

    pub fn assign(&self, entity: Entity, path_text: &str) -> RefMut<Mesh> {
        let mesh =
            self.resource_manager
            .get_gpu_mesh(path_text)
            .ok_or_else(|| format!("ERROR: Unable to assign mesh with uri {}", path_text))
            .unwrap(); // OK to panic here, indicates a bug in gameplay code.
        self.give_mesh(entity, mesh)
    }

    pub fn give_mesh(&self, entity: Entity, mesh: GLMeshData) -> RefMut<Mesh> {
        let shader = self.resource_manager.get_shader("shaders/forward_phong.glsl").unwrap();
        self.inner.assign(entity, Mesh {
            gl_mesh: mesh,
            shader: shader,
        })
    }

    pub fn iter(&self) -> Iter<Mesh> {
        self.inner.iter()
    }
}

impl ComponentManager for MeshManager {
    type Component = Mesh;

    fn destroy(&self, entity: Entity) {
        self.inner.destroy(entity);
    }
}
