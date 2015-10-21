use std::ops::{Deref, DerefMut};
use std::fmt::Debug;
use std::any::Any;

use ecs::*;

#[derive(Debug, Clone)]
pub struct SingletonComponentManager<T: Debug + Clone + Any>(T);

impl<T: Debug + Clone + Any> SingletonComponentManager<T> {
    pub fn new(data: T) -> SingletonComponentManager<T> {
        SingletonComponentManager(data)
    }
}

impl<T: Debug + Clone + Any> Deref for SingletonComponentManager<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Debug + Clone + Any> DerefMut for SingletonComponentManager<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Debug + Clone + Any> ComponentManager for SingletonComponentManager<T> {
    fn destroy_all(&self, _: Entity) {}
    fn destroy_marked(&mut self) {}
}
