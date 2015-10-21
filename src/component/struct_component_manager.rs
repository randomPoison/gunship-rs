use std::collections::{HashMap, HashSet};
use std::slice::Iter;
use std::cell::{RefCell, Ref, RefMut};
use std::any::Any;

use super::{EntityMap, EntitySet};

use ecs::{Entity, ComponentManager};

/// A default manager for component types that can be represented as a single struct.
#[derive(Debug, Clone)]
pub struct StructComponentManager<T: Clone + Any> {
    components: Vec<RefCell<T>>,
    entities: Vec<Entity>,
    indices: EntityMap<usize>,

    marked_for_destroy: RefCell<EntitySet>,
}

impl<T: Clone + Any> StructComponentManager<T> {
    pub fn new() -> StructComponentManager<T> {
        StructComponentManager {
            components: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::default(),

            marked_for_destroy: RefCell::new(HashSet::default()),
        }
    }

    pub fn assign(&mut self, entity: Entity, component: T) -> RefMut<T> {
        assert!(!self.indices.contains_key(&entity));

        let index = self.components.len();
        self.components.push(RefCell::new(component));
        self.entities.push(entity);
        self.indices.insert(entity, index);

        self.components[index].borrow_mut()
    }

    pub fn get(&self, entity: Entity) -> Option<Ref<T>> {
        if let Some(index) = self.indices.get(&entity) {
            Some(self.components[*index].borrow())
        } else {
            None
        }
    }

    pub fn get_mut(&self, entity: Entity) -> Option<RefMut<T>> {
        if let Some(index) = self.indices.get(&entity) {
            Some(self.components[*index].borrow_mut())
        } else {
            None
        }
    }

    pub fn components(&self) -> &[RefCell<T>] {
        &*self.components
    }

    pub fn entities(&self) -> &[Entity] {
        &*self.entities
    }

    pub fn iter(&self) -> ComponentIter<T> {
        ComponentIter {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn iter_mut(&self) -> ComponentIterMut<T> {
        ComponentIterMut {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn destroy_immediate(&mut self, entity: Entity) -> T {
        // Retrieve indices of removed entity and the one it's swapped with.
        let index = self.indices.remove(&entity).unwrap();

        // Remove transform and the associate entity.
        let removed_entity = self.entities.swap_remove(index);
        debug_assert!(removed_entity == entity);

        // Update the index mapping for the moved entity, but only if the one we removed
        // wasn't the only one in the row (or the last one in the row).
        if index != self.entities.len() {
            let moved_entity = self.entities[index];
            self.indices.insert(moved_entity, index);
        }

        // Defer removing the transform until the very end to avoid a bunch of memcpys.
        // Transform is a pretty fat struct so if we remove it, cache it to a variable,
        // and then return it at the end we wind up with 2 or 3 memcpys. Doing it all at
        // once at the end (hopefully) means only a single memcpy.
        self.components.swap_remove(index).into_inner()
    }
}

impl<T: Clone + Any> ComponentManager for StructComponentManager<T> {
    fn destroy_all(&self, entity: Entity) {
        if self.indices.contains_key(&entity) {
            self.marked_for_destroy.borrow_mut().insert(entity);
        }
    }

    fn destroy_marked(&mut self) {
        let mut marked_for_destroy = RefCell::new(HashSet::default());
        ::std::mem::swap(&mut marked_for_destroy, &mut self.marked_for_destroy);
        let mut marked_for_destroy = marked_for_destroy.into_inner();
        for entity in marked_for_destroy.drain() {
            self.destroy_immediate(entity);
        }
    }
}

pub struct ComponentIter<'a, T: 'a> {
    component_iter: Iter<'a, RefCell<T>>,
    entity_iter: Iter<'a, Entity>,
}

impl<'a, T: 'a + Clone> Iterator for ComponentIter<'a, T> {
    type Item = (Ref<'a, T>, Entity);

    fn next(&mut self) -> Option<(Ref<'a, T>, Entity)> {
        match self.component_iter.next() {
            None => None,
            Some(component) => Some((component.borrow(), *self.entity_iter.next().unwrap()))
        }
    }
}

pub struct ComponentIterMut<'a, T: 'a + Clone> {
    component_iter: Iter<'a, RefCell<T>>,
    entity_iter: Iter<'a, Entity>,
}


impl<'a, T: 'a + Clone> Iterator for ComponentIterMut<'a, T> {
    type Item = (RefMut<'a, T>, Entity);

    fn next(&mut self) -> Option<(RefMut<'a, T>, Entity)> {
        match self.component_iter.next() {
            None => None,
            Some(component) => Some((component.borrow_mut(), *self.entity_iter.next().unwrap()))
        }
    }
}
