use std::ops::{Deref, DerefMut};
use std::fmt::Debug;

use ecs::*;

#[derive(Debug, Clone)]
pub struct SingletonComponentManager<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone>(T);

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone> SingletonComponentManager<T> {
    pub fn new(data: T) -> SingletonComponentManager<T> {
        SingletonComponentManager(data)
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone> Deref for SingletonComponentManager<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone> DerefMut for SingletonComponentManager<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone> ComponentManager for SingletonComponentManager<T> {
    type Component = T;

    fn destroy(&self, _: Entity) {}
}
