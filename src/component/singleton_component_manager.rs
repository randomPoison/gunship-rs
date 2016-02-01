use ecs::*;
use engine::*;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    data: T,
    messages: RefCell<Vec<T::Message>>,
}

impl<T> SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    pub fn new(data: T) -> SingletonComponentManager<T> {
        SingletonComponentManager {
            data: data,
            messages: RefCell::new(Vec::new()),
        }
    }

    pub fn send_message<U: Into<T::Message>>(&self, message: U) {
        self.messages.borrow_mut().push(message.into());
    }
}

impl<T> Deref for SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> DerefMut for SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>> + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T, U> ComponentManagerBase for SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>, Message=U> + Debug + Clone + Default,
          U: Message<Target=T>,
{
    fn update(&mut self) {
        let mut messages = self.messages.borrow_mut();
        for message in messages.drain(..) {
            message.apply(&mut self.data);
        }
    }
}

impl<T, U> ComponentManager for SingletonComponentManager<T>
    where T: Component<Manager=SingletonComponentManager<T>, Message=U> + Debug + Clone + Default,
          U: Message<Target=T>,
{
    type Component = T;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(SingletonComponentManager::new(T::default()));
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        panic!("Singleton components do not need to be retreived, they can be derefenced from the manager");
    }

    fn destroy(&self, _: Entity) {}
}

// =======================================
// SINGLETON COMPONENT MANAGER WITH UPDATE
// =======================================

// TODO: Having a separate type for this won't be necessary once specialization is implemented.

#[derive(Debug, Clone)]
pub struct SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>> + ComponentUpdate + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    data: T,
    messages: RefCell<Vec<T::Message>>,
}

impl<T> SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>> + ComponentUpdate + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    pub fn new(data: T) -> SingletonComponentUpdateManager<T> {
        SingletonComponentUpdateManager {
            data: data,
            messages: RefCell::new(Vec::new()),
        }
    }

    pub fn send_message<U: Into<T::Message>>(&self, message: U) {
        self.messages.borrow_mut().push(message.into());
    }
}

impl<T> Deref for SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>> + ComponentUpdate + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> DerefMut for SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>> + ComponentUpdate + Debug + Clone + Default,
          T::Message: Message<Target=T>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T, U> ComponentManagerBase for SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>, Message=U> + ComponentUpdate + Debug + Clone + Default,
          U: Message<Target=T>,
{
    fn update(&mut self) {
        let mut messages = self.messages.borrow_mut();
        for message in messages.drain(..) {
            message.apply(&mut self.data);
        }
        self.data.update();
    }
}

impl<T, U> ComponentManager for SingletonComponentUpdateManager<T>
    where T: Component<Manager=SingletonComponentUpdateManager<T>, Message=U> + ComponentUpdate + Debug + Clone + Default,
          U: Message<Target=T>,
{
    type Component = T;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(SingletonComponentUpdateManager::new(T::default()));
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        panic!("Singleton components do not need to be retreived, they can be derefenced from the manager");
    }

    fn destroy(&self, _: Entity) {}
}
