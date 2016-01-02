use ecs::*;
use engine::*;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct SingletonComponentManager<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default>(T);

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default> SingletonComponentManager<T> {
    pub fn new(data: T) -> SingletonComponentManager<T> {
        SingletonComponentManager(data)
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default> Deref for SingletonComponentManager<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default> DerefMut for SingletonComponentManager<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default> ComponentManager for SingletonComponentManager<T> {
    type Component = T;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(Self::default());
    }

    fn destroy(&self, _: Entity) {}
}
