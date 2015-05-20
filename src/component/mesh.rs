use std::collections::HashMap;
use std::slice::Iter;
use std::rc::Rc;
use std::cell::RefCell;

use polygon::gl_render::GLMeshData;

use ecs::{Entity, ComponentManager};
use resource::ResourceManager;

pub type Mesh = GLMeshData;

pub struct MeshManager {
    resource_manager: Rc<RefCell<ResourceManager>>,
    meshes: Vec<GLMeshData>,
    entities: Vec<Entity>,
    indices: HashMap<Entity, usize>,
}

impl MeshManager {
    pub fn new(resource_manager: Rc<RefCell<ResourceManager>>) -> MeshManager {
        MeshManager {
            resource_manager: resource_manager,
            meshes: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn create(&mut self, entity: Entity, path_text: &str) -> &GLMeshData {
        assert!(!self.indices.contains_key(&entity));

        let index = self.meshes.len();
        self.meshes.push(self.resource_manager.borrow_mut().get_mesh(path_text));
        self.entities.push(entity);
        self.indices.insert(entity, index);
        &self.meshes[index]
    }

    pub fn meshes(&self) -> &Vec<GLMeshData> {
        &self.meshes
    }

    pub fn iter(&self) -> MeshIter {
        MeshIter {
            mesh_iter: self.meshes.iter(),
            entity_iter: self.entities.iter()
        }
    }
}

impl ComponentManager for MeshManager {
}

pub struct MeshIter<'a> {
    mesh_iter: Iter<'a, GLMeshData>,
    entity_iter: Iter<'a, Entity>,
}

impl<'a> Iterator for MeshIter<'a> {
    type Item = (&'a GLMeshData, Entity);

    fn next(&mut self) -> Option<(&'a GLMeshData, Entity)> {
        match self.mesh_iter.next() {
            None => None,
            Some(mesh) => Some((mesh, *self.entity_iter.next().unwrap()))
        }
    }
}
