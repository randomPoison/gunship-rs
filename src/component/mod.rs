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
use scene::Scene;
use self::struct_component_manager::StructComponentManager;
use std::ops::{Deref, DerefMut};

pub use self::singleton_component_manager::SingletonComponentManager;
pub use self::transform::{Transform, TransformManager, transform_update};
pub use self::camera::{Camera, CameraManager};
pub use self::mesh::{Mesh, MeshManager};
pub use self::light::{Light, LightManager, LightUpdateSystem};
pub use self::audio::{AudioSource, AudioSourceManager, AudioSystem};
pub use self::alarm::{AlarmId, AlarmManager, alarm_update};
pub use self::collider::{Collider, ColliderManager, CollisionSystem, bounding_volume, grid_collision};

#[derive(Debug, Clone)]
pub struct DefaultManager<T: Component + Clone>(StructComponentManager<T>);

impl<T: 'static + Component<Manager=DefaultManager<T>> + Clone> DefaultManager<T> {
    pub fn new() -> DefaultManager<T> {
        DefaultManager(StructComponentManager::new())
    }
}

impl<T: Component<Manager=DefaultManager<T>> + Clone> ComponentManager for DefaultManager<T> {
    type Component = T;

    fn register(scene: &mut Scene) {
        scene.register_manager(Self::new())
    }

    fn destroy(&self, entity: Entity) {
        self.0.destroy(entity);
    }
}

impl<T: Component + Clone> Deref for DefaultManager<T> {
    type Target = StructComponentManager<T>;

    fn deref(&self) -> &StructComponentManager<T> {
        &self.0
    }
}

impl<T: Component + Clone> DerefMut for DefaultManager<T> {
    fn deref_mut(&mut self) -> &mut StructComponentManager<T> {
        &mut self.0
    }
}
