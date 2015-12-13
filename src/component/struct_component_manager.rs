use collections::{EntityMap, EntitySet};
use ecs::{ComponentManager, Entity};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

const MAX_CAPACITY: usize = 1_000;

/// A default manager for component types that can be represented as a single struct.
#[derive(Debug, Clone)]
pub struct StructComponentManager<T: Clone + Any> {
    components: Vec<RefCell<Option<T>>>,
    entities: Vec<RefCell<Option<Entity>>>,
    indices: RefCell<EntityMap<usize>>,

    /// Tracks the maximum "count" of the components.
    ///
    /// This replaces the `.count()` value that would normally be gotten from the `Vec`, however we
    /// can't actually change the number of elements in the `Vec` (it all has to be immutable
    /// borrows), so we track it manually. This allows us to determine where to insert new
    /// components and which elements we need to iterate over.
    filled_count: Cell<usize>,
    entity_count: Cell<usize>,

    marked_for_destroy: RefCell<EntitySet>,
}

impl<T: Clone + Any> StructComponentManager<T> {
    pub fn new() -> StructComponentManager<T> {
        let mut manager = StructComponentManager::<T> {
            components: Vec::with_capacity(MAX_CAPACITY),
            entities: Vec::with_capacity(MAX_CAPACITY),
            indices: RefCell::new(HashMap::default()),

            filled_count: Cell::new(0),
            entity_count: Cell::new(0),

            marked_for_destroy: RefCell::new(HashSet::default()),
        };

        manager.components.resize(MAX_CAPACITY, RefCell::new(None));
        manager.entities.resize(MAX_CAPACITY, RefCell::new(None));

        manager
    }

    pub fn assign(&self, entity: Entity, component: T) -> RefMut<T> {
        debug_assert!(!self.indices.borrow().contains_key(&entity));

        let index = self.filled_count.get();
        debug_assert!(index < MAX_CAPACITY);

        // Write the new component data into the open slot.
        let mut component_slot = self.components[index].borrow_mut();
        debug_assert!(component_slot.is_none());

        *component_slot = Some(component);

        // Write entity to entities list.
        let mut entity_slot = self.entities[index].borrow_mut();
        debug_assert!(entity_slot.is_none());

        *entity_slot = Some(entity);

        // Add entity index to index map.
        let mut indices = self.indices.borrow_mut();
        indices.insert(entity, index);

        // Update filled count.
        self.filled_count.set(index + 1);
        self.entity_count.set(self.entity_count.get() + 1);

        RefMut(component_slot)
    }

    pub fn get(&self, entity: Entity) -> Option<Ref<T>> {
        if let Some(index) = self.indices.borrow().get(&entity) {
            Some(Ref(self.components[*index].borrow()))
        } else {
            None
        }
    }

    pub fn get_mut(&self, entity: Entity) -> Option<RefMut<T>> {
        if let Some(index) = self.indices.borrow().get(&entity) {
            Some(RefMut(self.components[*index].borrow_mut()))
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn iter_mut(&self) -> IterMut<T> {
        IterMut {
            component_iter: self.components.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    /// Returns the number of entities with a component associated.
    pub fn len(&self) -> usize {
        self.entity_count.get()
    }
}

impl<T: Clone + Any> ComponentManager for StructComponentManager<T> {
    fn destroy_all(&self, entity: Entity) {
        let maybe_index = self.indices.borrow_mut().remove(&entity);
        if let Some(index) = maybe_index {
            let mut component_slot = self.components[index].borrow_mut();
            let mut entity_slot = self.entities[index].borrow_mut();

            debug_assert!(component_slot.is_some());
            debug_assert!(entity_slot.is_some() && entity_slot.unwrap() == entity);

            *component_slot = None;
            *entity_slot = None;

            self.entity_count.set(self.entity_count.get() - 1);
        }
    }

    fn destroy_marked(&mut self) {
    }
}

pub struct Ref<'a, T: 'a>(::std::cell::Ref<'a, Option<T>>);

impl<'a, T: 'a> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

pub struct RefMut<'a, T: 'a>(::std::cell::RefMut<'a, Option<T>>);

impl<'a, T: 'a> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<'a, T: 'a> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

pub struct Iter<'a, T: 'a> {
    component_iter: ::std::slice::Iter<'a, RefCell<Option<T>>>,
    entity_iter: ::std::slice::Iter<'a, RefCell<Option<Entity>>>,
}

impl<'a, T: 'a + Clone> Iterator for Iter<'a, T> {
    type Item = (Ref<'a, T>, Entity);

    fn next(&mut self) -> Option<(Ref<'a, T>, Entity)> {
        loop {
            let next_entity = self.entity_iter.next();
            match self.component_iter.next() {
                None => return None,
                Some(maybe_component) => {
                    if maybe_component.borrow().is_none() {
                        continue;
                    }

                    return Some((Ref(maybe_component.borrow()), next_entity.unwrap().borrow().unwrap()));
                },
            }
        }
    }
}

pub struct IterMut<'a, T: 'a + Clone> {
    component_iter: ::std::slice::Iter<'a, RefCell<Option<T>>>,
    entity_iter: ::std::slice::Iter<'a, RefCell<Option<Entity>>>,
}

impl<'a, T: 'a + Clone> Iterator for IterMut<'a, T> {
    type Item = (RefMut<'a, T>, Entity);

    fn next(&mut self) -> Option<(RefMut<'a, T>, Entity)> {
        loop {
            let next_entity = self.entity_iter.next();
            match self.component_iter.next() {
                None => return None,
                Some(maybe_component) => {
                    if maybe_component.borrow().is_none() {
                        continue;
                    }

                    return Some((RefMut(maybe_component.borrow_mut()), next_entity.unwrap().borrow().unwrap()));
                },
            }
        }
    }
}
