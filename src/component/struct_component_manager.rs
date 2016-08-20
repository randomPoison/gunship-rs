use collections::{Array, EntityMap, EntitySet};
use ecs::*;
use scene::Scene;
use std::cell::RefCell;
use std::fmt::{Debug, Error, Formatter};
use std::intrinsics::type_name;
use std::ops::*;

const MAX_COMPONENTS: usize = 1_000;

struct MessageMap<T: Component>(EntityMap<Vec<T::Message>>);

impl<T: Component> MessageMap<T> {
    fn new() -> MessageMap<T> {
        MessageMap(EntityMap::default())
    }
}

impl<T: Component> Clone for MessageMap<T> {
    fn clone(&self) -> MessageMap<T> {
        MessageMap::new()
    }
}

impl<T: Component> Deref for MessageMap<T> {
    type Target = EntityMap<Vec<T::Message>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Component> DerefMut for MessageMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Component> Debug for MessageMap<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", unsafe { type_name::<Self>() })
    }
}

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
pub struct StructComponentManager<T>
    where T: Component + Clone + Debug,
          T::Message: Message<Target=T>,
{
    components: Array<T>,
    entities: Array<Entity>,
    indices: RefCell<EntityMap<usize>>,

    marked_for_destroy: RefCell<EntitySet>,
    messages: RefCell<MessageMap<T>>,
}

impl<T> StructComponentManager<T>
    where T: Component + Clone + Debug,
          T::Message: Message<Target=T>,
{
    pub fn new() -> StructComponentManager<T> {
        StructComponentManager {
            components: Array::new(MAX_COMPONENTS),
            entities: Array::new(MAX_COMPONENTS),
            indices: RefCell::new(EntityMap::default()),

            marked_for_destroy: RefCell::new(EntitySet::default()),
            messages: RefCell::new(MessageMap::new()),
        }
    }

    pub fn assign(&self, entity: Entity, component: T) -> &T {
        assert!(
            !self.indices.borrow().contains_key(&entity),
            "Component already assign to entity {:?}",
            entity);

        let index = self.components.len();
        self.components.push(component);
        self.entities.push(entity);
        self.indices.borrow_mut().insert(entity, index);

        &self.components[index]
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.indices
        .borrow()
        .get(&entity)
        .map(|index| &self.components[*index])
    }

    pub fn update(&mut self, _scene: &Scene, _delta: f32) {
        println!("StructComponentManager::update()");
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
        self.entities.len()
    }

    /// Passes a message to the component associated with the specified entity.
    pub fn send_message<M: Into<T::Message>>(&self, entity: Entity, message: M) {
        let mut messages = self.messages.borrow_mut();
        messages
        .entry(entity)
        .or_insert(Vec::new())
        .push(message.into());
    }

    /// Applies all pending messages to their target components.
    pub fn process_messages(&mut self) {
        let mut messages = self.messages.borrow_mut();
        for (entity, mut messages) in messages.drain() {
            if let Some(index) = self.indices.borrow().get(&entity) {
                let component = &mut self.components[*index];
                for message in messages.drain(..) {
                    message.apply(component);
                }
            } else {
                // TODO: Panic or error? That could probably be configured at runtime.
                panic!(
                    "Attempted to pass message to {} of {:?} which does not exist",
                    unsafe { type_name::<T>() },
                    entity);
            }
        }
    }
}

pub struct Iter<'a, T: 'a> {
    component_iter: ::std::slice::Iter<'a, T>,
    entity_iter: ::std::slice::Iter<'a, Entity>,
}

impl<'a, T: 'a + Component> Iterator for Iter<'a, T> {
    type Item = (&'a T, Entity);

    fn next(&mut self) -> Option<(&'a T, Entity)> {
        if let (Some(component), Some(entity)) = (self.component_iter.next(), self.entity_iter.next()) {
            Some((component, *entity))
        } else {
            None
        }
    }
}

pub struct IterMut<'a, T: 'a + Component> {
    component_iter: ::std::slice::IterMut<'a, T>,
    entity_iter: ::std::slice::Iter<'a, Entity>,
}

impl<'a, T: 'a + Component> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Entity);

    fn next(&mut self) -> Option<(&'a mut T, Entity)> {
        if let (Some(component), Some(entity)) = (self.component_iter.next(), self.entity_iter.next()) {
            Some((component, *entity))
        } else {
            None
        }
    }
}
