use std::collections::HashMap;
use std::slice::{Iter, IterMut};

use ecs::{Entity, ComponentManager};

/// A default implementation for a component manager that can be represented
/// as a single struct.
pub struct StructComponentManager<T> {
    components: Vec<T>,
    entities: Vec<Entity>,
    indices: HashMap<Entity, usize>,
}

impl<T> StructComponentManager<T> {
    pub fn new() -> StructComponentManager<T> {
        StructComponentManager {
            components: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn create(&mut self, entity: Entity, component: T) -> &mut T {
        assert!(!self.indices.contains_key(&entity));

        let index = self.components.len();
        self.components.push(component);
        self.entities.push(entity);
        self.indices.insert(entity, index);

        &mut self.components[index]
    }

    pub fn components(&self) -> &Vec<T> {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut Vec<T> {
        &mut self.components
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn iter(&self) -> ComponentIter<T> {
        ComponentIter {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> ComponentIterMut<T> {
        ComponentIterMut {
            component_iter: self.components.iter_mut(),
            entity_iter: self.entities.iter(),
        }
    }
}

impl<T> ComponentManager for StructComponentManager<T> {
}

pub struct ComponentIter<'a, T: 'a> {
    component_iter: Iter<'a, T>,
    entity_iter: Iter<'a, Entity>,
}

impl<'a, T: 'a> Iterator for ComponentIter<'a, T> {
    type Item = (&'a T, Entity);

    fn next(&mut self) -> Option<(&'a T, Entity)> {
        match self.component_iter.next() {
            None => None,
            Some(camera) => Some((camera, *self.entity_iter.next().unwrap()))
        }
    }
}

pub struct ComponentIterMut<'a, T: 'a> {
    component_iter: IterMut<'a, T>,
    entity_iter: Iter<'a, Entity>,
}


impl<'a, T: 'a> Iterator for ComponentIterMut<'a, T> {
    type Item = (&'a mut T, Entity);

    fn next(&mut self) -> Option<(&'a mut T, Entity)> {
        match self.component_iter.next() {
            None => None,
            Some(camera) => Some((camera, *self.entity_iter.next().unwrap()))
        }
    }
}
