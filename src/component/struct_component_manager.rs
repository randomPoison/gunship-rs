use collections::{EntityMap, EntitySet};
use ecs::*;
use std::cell::RefCell;

const MAX_PENDING: usize = 1_000;

/// A utilty on which to build other component managers.
///
/// `StructComponentManager` provides a default system for implementing a component manager for any
/// type that can be represented as a single struct. It handles the details of assigning component
/// data to an entity, retrieving that data, and destroying it. It also handles the details of
/// doing all of that through only shared references. `StructComponentManager` however does not
/// implement `ComponentManager` because it is meant to be reused within other managers that want
/// to wrap extra behavior around the default management style. `DefaultManager` is a basic wrapper
/// around `StructComponentManager` that implements `ComponentManager` and should be used as the
/// default component manager when no special handling is needed.
#[derive(Debug, Clone)]
pub struct StructComponentManager<T: Clone> {
    components: Vec<T>,
    entities: Vec<Entity>,
    indices: EntityMap<usize>,

    // TODO: Convert to a non-resizable dynamicially allocated array.
    new_components: Vec<(Entity, T)>,
    marked_for_destroy: RefCell<EntitySet>,
}

impl<T: Clone> StructComponentManager<T> {
    pub fn new() -> StructComponentManager<T> {
        StructComponentManager {
            components: Vec::new(),
            entities: Vec::new(),
            indices: EntityMap::default(),

            new_components: Vec::with_capacity(MAX_PENDING),
            marked_for_destroy: RefCell::new(EntitySet::default()),
        }
    }

    pub fn assign(&mut self, entity: Entity, component: T) -> &mut T {
        debug_assert!(
            !self.indices.contains_key(&entity),
            "Component already assign to entity {:?}",
            entity);
        debug_assert!(
            self.new_components.len() <= MAX_PENDING,
            "Maximum pending components reached, maybe don't try to create more than {} components a frame",
            MAX_PENDING);

        let index = self.new_components.len();

        self.new_components.push((entity, component));
        self.indices.insert(entity, index);

        &mut self.new_components[index].1
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        if let Some(index) = self.indices.get(&entity) {
            Some(&self.components[*index])
        } else {
            None
        }
    }

    pub fn destroy(&self, entity: Entity) {
        self.marked_for_destroy.borrow_mut().insert(entity);
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            component_iter: self.components.iter_mut(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len() + self.new_components.len()
    }
}

pub struct Iter<'a, T: 'a> {
    component_iter: ::std::slice::Iter<'a, T>,
    entity_iter: ::std::slice::Iter<'a, Entity>,
}

impl<'a, T: 'a + Clone> Iterator for Iter<'a, T> {
    type Item = (&'a T, Entity);

    fn next(&mut self) -> Option<(&'a T, Entity)> {
        if let (Some(component), Some(entity)) = (self.component_iter.next(), self.entity_iter.next()) {
            Some((component, *entity))
        } else {
            None
        }
    }
}

pub struct IterMut<'a, T: 'a + Clone> {
    component_iter: ::std::slice::IterMut<'a, T>,
    entity_iter: ::std::slice::Iter<'a, Entity>,
}

impl<'a, T: 'a + Clone> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Entity);

    fn next(&mut self) -> Option<(&'a mut T, Entity)> {
        if let (Some(component), Some(entity)) = (self.component_iter.next(), self.entity_iter.next()) {
            Some((component, *entity))
        } else {
            None
        }
    }
}
