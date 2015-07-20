use std::collections::HashMap;
use std::any::{Any, TypeId};
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
use component::{TransformManager, CameraManager, MeshManager, LightManager, AudioSourceManager, AlarmManager};
use resource::ResourceManager;

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
#[derive(Debug)]
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    component_managers: HashMap<TypeId, RefCell<Box<Any>>>,
    /// This value is only needed in debug builds.
    manager_id_by_name: HashMap<String, TypeId>,
    pub input: Input,
    pub audio_source: AudioSource,
}

impl Scene {
    pub fn new(resource_manager: &Rc<RefCell<ResourceManager>>, audio_source: AudioSource) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            component_managers: HashMap::new(),
            manager_id_by_name: HashMap::new(),
            input: Input::new(),
            audio_source: audio_source,
        };

        scene.register_manager(TransformManager::new());
        scene.register_manager(CameraManager::new());
        scene.register_manager(LightManager::new());
        scene.register_manager(MeshManager::new(resource_manager.clone()));
        scene.register_manager(AudioSourceManager::new(resource_manager.clone()));
        scene.register_manager(AlarmManager::new());

        scene
    }

    pub fn clone(&self, resource_manager: &Rc<RefCell<ResourceManager>>) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(self.entity_manager.borrow().clone()),
            component_managers: HashMap::new(),
            manager_id_by_name: HashMap::new(),
            input: self.input.clone(),
            audio_source: self.audio_source.clone(),
        };

        // Reload internal component managers.
        scene.register_manager(self.get_manager_by_name::<TransformManager>().clone());
        scene.register_manager(self.get_manager_by_name::<CameraManager>().clone());
        scene.register_manager(self.get_manager_by_name::<LightManager>().clone());
        scene.register_manager(self.get_manager_by_name::<MeshManager>().clone(resource_manager.clone()));
        scene.register_manager(self.get_manager_by_name::<AudioSourceManager>().clone(resource_manager.clone()));
        scene.register_manager(self.get_manager_by_name::<AlarmManager>().clone());

        scene
    }

    pub fn register_manager<T: Any + ComponentManager>(&mut self, manager: T) {
        let manager_id = TypeId::of::<T>();
        assert!(!self.component_managers.contains_key(&manager_id),
                "Manager {} with ID {:?} already registered", type_name::<T>(), manager_id);

        self.component_managers.insert(manager_id, RefCell::new(Box::new(manager)));

        // TODO: Only do this when hotloading support is enabled.
        self.manager_id_by_name.insert(type_name::<T>().into(), manager_id);
    }

    pub fn get_manager<T: Any + ComponentManager>(&self) -> ManagerRef<T> {
        let manager_id = TypeId::of::<T>();
        let manager = self.component_managers
            .get(&manager_id)
            .expect(&format!("Tried to retrieve manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));

        ManagerRef {
            manager: manager.borrow(),
            _phantom: PhantomData,
        }
    }

    pub fn get_manager_mut<T: Any + ComponentManager>(&self) -> ManagerRefMut<T> {
        let manager_id = TypeId::of::<T>();
        let manager = self.component_managers
            .get(&manager_id)
            .expect(&format!("Tried to retrieve manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));

        ManagerRefMut {
            manager: manager.borrow_mut(),
            _phantom: PhantomData,
        }
    }

    pub fn remove_manager<T: Any + ComponentManager>(&mut self) -> Box<T> {
        let manager_id = TypeId::of::<T>();
        let manager = self.component_managers
            .remove(&manager_id)
            .expect(&format!("Tried to remove manager {} with ID {:?} but none exists", type_name::<T>(), manager_id));

        // TODO: Only do this when hotloading is enabled.
        self.manager_id_by_name.remove(type_name::<T>().into());

        manager.into_inner().downcast().unwrap()
    }

    pub fn create_entity(&self) -> Entity {
        self.entity_manager.borrow_mut().create()
    }

    /// TODO: We don't need this if hotloading isn't enabled.
    /// TODO: Allow this to handle reloading managers that are new by returning an Option<&T>.
    pub fn get_manager_by_name<T: Any + ComponentManager>(&self) -> &T {
        let manager_id = self.manager_id_by_name
            .get(type_name::<T>())
            .expect(&format!("Tried to remove manager {} by name but none exists", type_name::<T>()));

        let manager = self.component_managers
            .get(&manager_id)
            .expect(&format!("Tried to remove manager {} by name with ID {:?} but none exists", type_name::<T>(), manager_id));

        // Perform an unchecked downcast. We know this works because we stored the manager
        // by its type, but we can't use `Any::downcast_ref()` because the type id will be
        // different across different DLLs.
        unsafe {
            // Get the raw representation of the trait object.
            let to: TraitObject = mem::transmute(manager.borrow().deref().deref());

            // Extract the data pointer.
            mem::transmute(to.data)
        }
    }
}

fn type_name<T>() -> &'static str {
    unsafe {
        intrinsics::type_name::<T>()
    }
}

pub struct ManagerRef<'a, T: Any + ComponentManager> {
    manager: Ref<'a, Box<Any>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: Any + ComponentManager> Deref for ManagerRef<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        self.manager.downcast_ref().unwrap()
    }
}

pub struct ManagerRefMut<'a, T: Any + ComponentManager> {
    manager: RefMut<'a, Box<Any>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: Any + ComponentManager> Deref for ManagerRefMut<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        self.manager.downcast_ref().unwrap()
    }
}

impl<'a, T: Any + ComponentManager> DerefMut for ManagerRefMut<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.manager.downcast_mut().unwrap()
    }
}
