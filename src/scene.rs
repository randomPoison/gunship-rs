use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::intrinsics;

use bs_audio::AudioSource;

use ecs::{EntityManager, ComponentManager};
use input::Input;
use super::component::{TransformManager, CameraManager, MeshManager, LightManager, AudioSourceManager};
use resource::ResourceManager;

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
pub struct Scene {
    pub entity_manager: EntityManager,
    component_managers: Vec<Rc<RefCell<Box<Any>>>>,
    component_indices: HashMap<TypeId, usize>,
    pub input: Input,
    pub audio_source: AudioSource,
}

impl Scene {
    pub fn new(resource_manager: &Rc<RefCell<ResourceManager>>, audio_source: AudioSource) -> Scene {
        let mut scene = Scene {
            entity_manager: EntityManager::new(),
            component_managers: Vec::new(),
            component_indices: HashMap::new(),
            input: Input::new(),
            audio_source: audio_source,
        };

        scene.register_manager(Box::new(TransformManager::new()));
        scene.register_manager(Box::new(CameraManager::new()));
        scene.register_manager(Box::new(MeshManager::new(resource_manager.clone())));
        scene.register_manager(Box::new(LightManager::new()));
        scene.register_manager(Box::new(AudioSourceManager::new(resource_manager.clone())));

        scene
    }

    pub fn register_manager<T: Any + ComponentManager>(&mut self, manager: Box<T>) {
        let manager_id = TypeId::of::<T>();
        assert!(!self.component_indices.contains_key(&manager_id),
                "Manager {} with ID {:?} already registered", type_name::<T>(), manager_id);

        let index = self.component_managers.len();
        self.component_managers.push(Rc::new(RefCell::new(manager)));
        self.component_indices.insert(manager_id, index);
    }

    pub fn get_manager<T: Any + ComponentManager>(&self) -> ManagerHandle<T> {
        let manager_id = TypeId::of::<T>();
        let index = *self.component_indices
            .get(&manager_id)
            .expect(&format!("Tried to retrieve manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));
        let manager_clone = self.component_managers[index].clone();
        ManagerHandle::new(manager_clone)
    }
}

fn type_name<T>() -> &'static str {
    unsafe {
        intrinsics::type_name::<T>()
    }
}

#[derive(Clone)]
pub struct ManagerHandle<T: Any + ComponentManager> {
    manager: Rc<RefCell<Box<Any>>>,
    _type: PhantomData<T>,
}

impl<'a, T: Any + ComponentManager> ManagerHandle<T> {
    pub fn new(manager: Rc<RefCell<Box<Any>>>) -> ManagerHandle<T> {
        ManagerHandle {
            manager: manager,
            _type: PhantomData,
        }
    }

    pub fn get(&'a mut self) -> ManagerRef<'a, T> {
        ManagerRef {
            borrow: (*self.manager).borrow_mut(),
            _type: PhantomData,
        }
    }
}

pub struct ManagerRef<'a, T: Any + ComponentManager> {
    borrow: RefMut<'a, Box<Any>>,
    _type: PhantomData<T>,
}

impl<'a, T: Any + ComponentManager> Deref for ManagerRef<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        self.borrow.downcast_ref().unwrap()
    }
}

impl<'a, T: Any + ComponentManager> DerefMut for ManagerRef<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.borrow.downcast_mut().unwrap()
    }
}
