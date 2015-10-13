use std::collections::VecDeque;
use std::fmt;

use scene::Scene;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity(u32);

#[derive(Debug, Clone)]
pub struct EntityManager {
    entities: Vec<Entity>,
    recycled_entities: VecDeque<Entity>,
    marked_for_destroy: Vec<Entity>,
    id_counter: u32
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            entities: Vec::new(),
            recycled_entities: VecDeque::new(),
            marked_for_destroy: Vec::new(),
            id_counter: 1
        }
    }

    pub fn create(&mut self) -> Entity {
        if let Some(entity) = self.recycled_entities.pop_front() {
            return entity;
        }

        let entity = Entity(self.id_counter);
        self.id_counter += 1;
        self.entities.push(entity);
        entity
    }

    pub fn mark_for_destroy(&mut self, entity: Entity) {
        debug_assert!(!self.marked_for_destroy.contains(&entity), "Can't mark an entity for destruction more than once");
        self.marked_for_destroy.push(entity);
    }

    pub fn destroy_marked(&mut self) {
        for entity in self.marked_for_destroy.drain(0..) {
            debug_assert!(!self.recycled_entities.iter().any(|existing| &entity == existing), "Trying to recycle entity {:?} but it is already recycled");
            self.recycled_entities.push_back(entity)
        }
    }

    pub fn destroy_immediate(&mut self, entity: Entity) {
        debug_assert!(!self.recycled_entities.iter().any(|existing| &entity == existing), "Trying to recycle entity {:?} but it is already recycled");

        self.recycled_entities.push_back(entity);
    }
}

pub trait System {
    fn update(&mut self, scene: &Scene, delta: f32);
}

impl<T: ?Sized> System for T where T: FnMut(&Scene, f32) {
    fn update(&mut self, scene: &Scene, delta: f32) {
        self.call_mut((scene, delta));
    }
}

pub trait ComponentManager: ::std::any::Any {
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
