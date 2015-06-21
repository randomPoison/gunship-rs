use std::collections::HashMap;
use std::slice::Iter;
use std::cell::{RefCell, Ref, RefMut};

use ecs::{Entity, ComponentManager};

/// A default implementation for a component manager that can be represented
/// as a single struct.
#[derive(Clone)]
pub struct StructComponentManager<T: Clone> {
    components: Vec<RefCell<T>>,
    entities: Vec<Entity>,
    indices: HashMap<Entity, usize>,
}

impl<T: Clone> StructComponentManager<T> {
    pub fn new() -> StructComponentManager<T> {
        StructComponentManager {
            components: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
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

    pub fn get(&self, entity: Entity) -> Ref<T> {
        assert!(self.indices.contains_key(&entity));

        let index = *self.indices.get(&entity).unwrap();
        self.components[index].borrow()
    }

    pub fn get_mut(&self, entity: Entity) -> RefMut<T> {
        assert!(self.indices.contains_key(&entity));

        let index = *self.indices.get(&entity).unwrap();
        self.components[index].borrow_mut()
    }

    pub fn components(&self) -> &Vec<RefCell<T>> {
        &self.components
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

    pub fn iter_mut(&self) -> ComponentIterMut<T> {
        ComponentIterMut {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }
}

impl<T: Clone> ComponentManager for StructComponentManager<T> {
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
