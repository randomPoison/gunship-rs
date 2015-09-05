use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::intrinsics;
use std::mem;
use std::raw::TraitObject;
use std::marker::PhantomData;

use bs_audio::AudioSource;

use ecs::{Entity, EntityManager, ComponentManager};
use input::Input;
use component::{TransformManager, CameraManager, MeshManager, LightManager, AudioSourceManager,
                AlarmManager, ColliderManager, bounding_volume};
use resource::ResourceManager;

#[cfg(not(feature = "hotloading"))]
type ManagerId = ::std::any::TypeId;

#[cfg(feature = "hotloading")]
type ManagerId = &'static str;

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
#[derive(Debug)]
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    component_managers: HashMap<ManagerId, RefCell<Box<ComponentManager>>>,
    pub input: Input,
    pub audio_source: AudioSource,
    resource_manager: Rc<ResourceManager>,
}

impl Scene {
    pub fn new(resource_manager: &Rc<ResourceManager>, audio_source: AudioSource) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            component_managers: HashMap::new(),
            input: Input::new(),
            audio_source: audio_source,
            resource_manager: resource_manager.clone(),
        };

        scene.register_manager(TransformManager::new());
        scene.register_manager(CameraManager::new());
        scene.register_manager(LightManager::new());
        scene.register_manager(MeshManager::new(resource_manager.clone()));
        scene.register_manager(AudioSourceManager::new(resource_manager.clone()));
        scene.register_manager(AlarmManager::new());
        scene.register_manager(ColliderManager::new());
        scene.register_manager(bounding_volume::BoundingVolumeManager::new());

        scene
    }

    pub fn clone(&self, resource_manager: &Rc<ResourceManager>) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(self.entity_manager.borrow().clone()),
            component_managers: HashMap::new(),
            input: self.input.clone(),
            audio_source: self.audio_source.clone(),
            resource_manager: resource_manager.clone(),
        };

        // Reload internal component managers.
        scene.reload_manager::<TransformManager>(self);
        scene.reload_manager::<CameraManager>(self);
        scene.reload_manager::<LightManager>(self);
        scene.reload_manager::<AlarmManager>(self);
        scene.reload_manager::<ColliderManager>(self);
        scene.reload_manager::<bounding_volume::BoundingVolumeManager>(self);
        scene.register_manager(self.get_manager::<MeshManager>().clone(resource_manager.clone()));
        scene.register_manager(self.get_manager::<AudioSourceManager>().clone(resource_manager.clone()));

        scene
    }

    pub fn register_manager<T: ComponentManager>(&mut self, manager: T) {
        let manager_id = manager_id::<T>();
        assert!(!self.component_managers.contains_key(&manager_id),
                "Manager {} with ID {:?} already registered", type_name::<T>(), manager_id);

        self.component_managers.insert(manager_id, RefCell::new(Box::new(manager)));
    }

    pub fn get_manager<T: ComponentManager>(&self) -> ManagerRef<T> {
        let manager_id = manager_id::<T>();
        let manager = self.component_managers
            .get(&manager_id)
            .expect(&format!("Tried to retrieve manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));

        ManagerRef {
            manager: manager.borrow(),
            _phantom: PhantomData,
        }
    }

    pub fn get_manager_mut<T: ComponentManager>(&self) -> ManagerRefMut<T> {
        let manager_id = manager_id::<T>();
        let manager = self.component_managers
            .get(&manager_id)
            .expect(&format!("Tried to retrieve manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));

        ManagerRefMut {
            manager: manager.borrow_mut(),
            _phantom: PhantomData,
        }
    }

    pub fn reload_manager<T: ComponentManager + Clone>(&mut self, old_scene: &Scene) {
        self.register_manager(old_scene.get_manager::<T>().clone());
    }

    pub fn create_entity(&self) -> Entity {
        self.entity_manager.borrow_mut().create()
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        &*self.resource_manager
    }

    /// Instantiates an instance of the model in the scene, returning the root entity.
    pub fn instantiate_model(&self, resource: &str) -> Entity {
        self.resource_manager.clone().instantiate_model(resource, self).unwrap()
    }

    pub fn destroy_entity(&self, entity: Entity) {
        for (_, manager) in self.component_managers.iter() {
            manager.borrow_mut().destroy_all(entity);
        }

        self.entity_manager.borrow_mut().destroy(entity);
    }

    pub fn destroy_marked(&self) {
        for (_, manager) in self.component_managers.iter() {
            manager.borrow_mut().destroy_marked();
        }
    }
}

#[cfg(not(feature = "hotloading"))]
fn manager_id<T: ComponentManager>() -> ManagerId {
    ::std::any::TypeId::of::<T>()
}

#[cfg(feature = "hotloading")]
fn manager_id<T: ComponentManager>() -> ManagerId {
    let full_name = type_name::<T>();

    // Find first occurrence of '<' character since we know the start of the proper name has to
    // be before that.
    let sub_str = match full_name.find("<") {
        Some(index) => &full_name[0..index],
        None => full_name ,
    };

    let slice_index = match sub_str.rfind("::") {
        Some(last_index) => last_index + 2,
        None => 0,
    };

    &full_name[slice_index..]
}

fn type_name<T>() -> &'static str {
    unsafe {
        intrinsics::type_name::<T>()
    }
}

pub struct ManagerRef<'a, T: ComponentManager> {
    manager: Ref<'a, Box<ComponentManager>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: ComponentManager> Deref for ManagerRef<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        unsafe { downcast_manager(self.manager.deref().deref()) }
    }
}

pub struct ManagerRefMut<'a, T: ComponentManager> {
    manager: RefMut<'a, Box<ComponentManager>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: ComponentManager> Deref for ManagerRefMut<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        unsafe { downcast_manager(self.manager.deref().deref()) }
    }
}

impl<'a, T: ComponentManager> DerefMut for ManagerRefMut<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        let manager = self.manager.deref_mut().deref_mut();
        unsafe { downcast_manager_mut(manager) }
    }
}

/// Performs an unchecked downcast from the `ComponentManager` trait object to the concrete type.
unsafe fn downcast_manager<'a, T: ComponentManager>(manager: &'a ComponentManager) -> &'a T {
    // Get the raw representation of the trait object.
    let to: TraitObject = mem::transmute(manager);

    // Extract the data pointer.
    mem::transmute(to.data)
}

unsafe fn downcast_manager_mut<'a, T: ComponentManager>(manager: &'a mut ComponentManager) -> &'a mut T {
    // Get the raw representation of the trait object.
    let to: TraitObject = mem::transmute(manager);

    // Extract the data pointer.
    mem::transmute(to.data)
}
