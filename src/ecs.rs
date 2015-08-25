use std::fmt;

use scene::Scene;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity {
    id: u32
}

#[derive(Debug, Clone)]
pub struct EntityManager {
    entities: Vec<Entity>,
    id_counter: u32
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            entities: Vec::new(),
            id_counter: 1
        }
    }

    pub fn create(&mut self) -> Entity {
        let entity = Entity {
            id: self.id_counter
        };
        self.id_counter += 1;
        self.entities.push(entity);
        entity
    }
}

pub trait System {
    fn update(&mut self, scene: &Scene, delta: f32);
}

pub trait ComponentManager {
    /// Destroy all component data associated with the entity.
    fn destroy_all(&self, Entity);

    /// Destroy all previously marked components.
    fn destroy_marked(&mut self);
}

impl ::std::fmt::Debug for ComponentManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("ComponentManager")
    }
}
