pub mod transform;
pub mod camera;
pub mod mesh;
pub mod light;
pub mod audio;
pub mod alarm;
pub mod singleton_component_manager;
pub mod struct_component_manager;
pub mod collider;

use ecs::*;
use engine::*;
use self::struct_component_manager::StructComponentManager;
use std::boxed::FnBox;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub use self::singleton_component_manager::SingletonComponentManager;
pub use self::transform::{Transform, TransformManager};
pub use self::camera::{Camera, CameraManager};
pub use self::mesh::{Mesh, MeshManager};
pub use self::light::{Light, LightManager};
pub use self::audio::{AudioSource, AudioSourceManager, AudioSystem};
pub use self::alarm::{AlarmId, AlarmManager, alarm_update};
pub use self::collider::{Collider, ColliderManager, CollisionSystem, bounding_volume, grid_collision};

#[derive(Debug, Clone)]
pub struct DefaultManager<T>(StructComponentManager<T>)
    where T: Component + Clone + Debug,
          T::Message: Message<Target=T>;

impl<T> DefaultManager<T>
    where T: Component<Manager=DefaultManager<T>> + Clone + Debug,
          T::Message: Message<Target=T>,
{
    pub fn new() -> DefaultManager<T> {
        DefaultManager(StructComponentManager::new())
    }
}

impl<T> ComponentManagerBase for DefaultManager<T>
    where T: Component<Manager=DefaultManager<T>> + Clone + Debug,
          T::Message: Message<Target=T>,
{
    fn update(&mut self) {
        self.0.process_messages();
    }
}

impl<T> ComponentManager for DefaultManager<T>
    where T: Component<Manager=DefaultManager<T>> + Clone + Debug,
          T::Message: Message<Target=T>,
{
    type Component = T;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(Self::new());
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.0.get(entity)
    }

    fn destroy(&self, entity: Entity) {
        self.0.destroy(entity);
    }
}

impl<T> Deref for DefaultManager<T>
    where T: Component<Manager=DefaultManager<T>> + Clone + Debug,
          T::Message: Message<Target=T>,
{
    type Target = StructComponentManager<T>;

    fn deref(&self) -> &StructComponentManager<T> {
        &self.0
    }
}

impl<T> DerefMut for DefaultManager<T>
    where T: Component<Manager=DefaultManager<T>> + Clone + Debug,
          T::Message: Message<Target=T>,
{
    fn deref_mut(&mut self) -> &mut StructComponentManager<T> {
        &mut self.0
    }
}

pub struct DefaultMessage<T>(Box<FnBox(&mut T)>)
    where T: Component;

impl<T: Component<Message=DefaultMessage<T>>> Message for DefaultMessage<T> {
    type Target = T;

    fn apply(self, component: &mut T) {
        let inner = self.0;
        inner.call_once((component,));
    }
}

impl<T, U> From<U> for DefaultMessage<T>
    where T: Component,
          U: 'static + FnOnce(&mut T),
{
    fn from(callback: U) -> DefaultMessage<T> {
        DefaultMessage(Box::new(callback))
    }
}
