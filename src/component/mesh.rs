use std::collections::HashMap;
use std::slice::Iter;
use std::rc::Rc;

use polygon::gl_render::GLMeshData;

use ecs::{Entity, ComponentManager};
use resource::ResourceManager;

pub type Mesh = GLMeshData;

pub struct MeshManager {
    resource_manager: Rc<ResourceManager>,
    meshes: Vec<GLMeshData>,
    entities: Vec<Entity>,
    indices: HashMap<Entity, usize>,
}

impl MeshManager {
    pub fn new(resource_manager: Rc<ResourceManager>) -> MeshManager {
        MeshManager {
            resource_manager: resource_manager,
            meshes: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn clone(&self, resource_manager: Rc<ResourceManager>) -> MeshManager {
        MeshManager {
            resource_manager: resource_manager,
            meshes: self.meshes.clone(),
            entities: self.entities.clone(),
            indices: self.indices.clone(),
        }
    }

    pub fn assign(&mut self, entity: Entity, path_text: &str) -> &GLMeshData {
        let mesh = self.resource_manager.get_mesh(path_text).unwrap();
        self.give_mesh(entity, mesh)
    }

    pub fn give_mesh(&mut self, entity: Entity, mesh: GLMeshData) -> &GLMeshData {
        debug_assert!(!self.indices.contains_key(&entity));

        let index = self.meshes.len();
        self.meshes.push(mesh);
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

    pub fn destroy_immediate(&mut self, entity: Entity) {
        let index = self.indices.remove(&entity)
                    .expect("Could not destroy mesh component because none is associated with the entity");
        self.meshes.swap_remove(index);
        let removed_entity = self.entities.swap_remove(index);
        assert_eq!(removed_entity, entity);

        if self.meshes.len() > 0 {
            let moved_entity = self.entities[index];
            self.indices.insert(moved_entity, index);
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
