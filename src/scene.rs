use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use ecs::{EntityManager, ComponentManager};
use input::Input;
use super::component::{TransformManager, CameraManager, MeshManager};
use resource::ResourceManager;

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
pub struct Scene {
    pub entity_manager: EntityManager,
    pub transform_manager: TransformManager,
    pub camera_manager: CameraManager,
    pub mesh_manager: MeshManager,
    components: Vec<Rc<RefCell<Box<Any>>>>,
    component_indices: HashMap<TypeId, usize>,
    pub input: Input,
}

impl Scene {
    pub fn new(resource_manager: Rc<RefCell<ResourceManager>>) -> Scene {
        Scene {
            entity_manager: EntityManager::new(),
            transform_manager: TransformManager::new(),
            camera_manager: CameraManager::new(),
            mesh_manager: MeshManager::new(resource_manager),
            components: Vec::new(),
            component_indices: HashMap::new(),
            input: Input::new(),
        }
    }

    pub fn register_manager<T: Any + ComponentManager>(&mut self, manager: Box<T>) {
        let manager_id = TypeId::of::<T>();
        assert!(!self.component_indices.contains_key(&manager_id));

        let index = self.components.len();
        self.components.push(Rc::new(RefCell::new(manager)));
        self.component_indices.insert(manager_id, index);
    }

    pub fn get_manager_mut<T: Any + ComponentManager>(&self) -> ManagerHandle<T> {
        let manager_id = TypeId::of::<T>();

        let index = *self.component_indices
            .get(&manager_id).expect("Scene must have the specified manager.");
        let manager_clone = self.components[index].clone();
        ManagerHandle::new(manager_clone)
    }
}

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
