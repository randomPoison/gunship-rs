use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::intrinsics;
use std::mem;
use std::raw::TraitObject;

use bs_audio::AudioSource;

use ecs::*;
use input::Input;
use component::{TransformManager, CameraManager, MeshManager, LightManager, AudioSourceManager,
                AlarmManager, ColliderManager};
use resource::ResourceManager;

#[cfg(not(feature = "hotloading"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ManagerId(u64);

#[cfg(feature = "hotloading")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ManagerId(&'static str);

impl ManagerId {
    #[cfg(not(feature = "hotloading"))]
    fn of<T: ComponentManager>() -> ManagerId {
        unsafe { ManagerId(intrinsics::type_id::<T>()) }
    }

    /// Two cases:
    ///
    /// - No template (e.g. `foo::bar::TransformManager`) just remove path (becomes `TransformManager`).
    /// - Template (e.g. `foo::bar::Manager<foo::bar::Foo>`) innermost type without leading path
    ///   (becomes `Foo`).
    #[cfg(feature = "hotloading")]
    fn of<T: ComponentManager>() -> ManagerId {
        let full_name = type_name::<T>();

        // Find first occurrence of '<' character since we know the start of the proper name has to
        // be before that.
        let sub_str = match full_name.find('<') {
            Some(index) => &full_name[index + 1..full_name.len() - 1], // foo::Foo<foo::Bar> => foo::Bar
            None => full_name,                                         // foo::Foo => foo::Foo
        };

        let slice_index = match sub_str.rfind("::") {
            Some(last_index) => last_index + 2,
            None => 0,
        };

        ManagerId(&sub_str[slice_index..])
    }
}

#[cfg(not(feature = "hotloading"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentId(u64);

#[cfg(feature = "hotloading")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentId(String);

impl ComponentId {
    #[cfg(not(feature="hotloading"))]
    pub fn of<T: 'static>() -> ComponentId {
        ComponentId(unsafe { ::std::intrinsics::type_id::<T>() })
    }

    #[cfg(feature="hotloading")]
    pub fn of<T: 'static>() -> ComponentId {
        ComponentId(String::from(unsafe { ::std::intrinsics::type_name::<T>() }))
    }
}

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    component_managers: HashMap<ManagerId, Box<ComponentManager<Component=()>>>,
    component_map: HashMap<ComponentId, ManagerId>,
    pub input: Input,
    pub audio_source: AudioSource,
    resource_manager: Rc<ResourceManager>,
}

impl Scene {
    pub fn new(resource_manager: &Rc<ResourceManager>, audio_source: AudioSource) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            component_managers: HashMap::new(),
            component_map: HashMap::new(),
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

        scene
    }

    pub fn clone(&self, resource_manager: &Rc<ResourceManager>) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(self.entity_manager.borrow().clone()),
            component_managers: HashMap::new(),
            component_map: HashMap::new(),
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
        scene.register_manager(self.get_manager::<MeshManager>().clone(resource_manager.clone()));
        scene.register_manager(self.get_manager::<AudioSourceManager>().clone(resource_manager.clone()));

        scene
    }

    pub fn register_manager<T: ComponentManager<Component=C>, C>(&mut self, manager: T) {
        let manager_id = ManagerId::of::<T>();
        let component_id = ComponentId::of::<T>();
        assert!(
            !self.component_managers.contains_key(&manager_id),
            "Manager {} with ID {:?} already registered", type_name::<T>(), manager_id);
        assert!(
            !self.component_map.contains_key(&component_id),
            "Manager already registered for component {}", type_name::<C>());

        // Box the manager as a trait object to construct the data and vtable pointers.
        let boxed_manager = Box::new(manager) as Box<ComponentManager<Component=C>>;

        // Transmute to raw trait object to throw away type information so that we can store it
        // in the type map.
        let boxed_trait_object = unsafe { mem::transmute(boxed_manager) };

        // Add the manager to the type map and the component id to the component map.
        self.component_managers.insert(manager_id.clone(), boxed_trait_object);
        self.component_map.insert(component_id, manager_id);
    }

    pub fn get_manager<T: ComponentManager>(&self) -> &T {
        let manager_id = ManagerId::of::<T>();
        let trait_object = match self.component_managers.get(&manager_id) {
            Some(trait_object) => &**trait_object,
            None => panic!(
                "Tried to retrieve manager {} with ID {:?} but none exists",
                type_name::<T>(),
                manager_id),
        };

        unsafe { downcast_manager(trait_object) }
    }

    // FIXME: DANGER! DANGER! VERY BAD!
    pub fn get_manager_mut<T: ComponentManager>(&self) -> &mut T {
        let manager_id = ManagerId::of::<T>();
        let trait_object = match self.component_managers.get(&manager_id) {
            Some(trait_object) => &**trait_object,
            None => panic!(
                "Tried to retrieve manager {} with ID {:?} but none exists",
                type_name::<T>(),
                manager_id),
        };

        unsafe { downcast_manager_mut(trait_object) }
    }

    pub fn get_manager_for<C: 'static>(&self) -> &ComponentManager<Component=C> {
        let component_id = ComponentId::of::<C>();
        let manager_id = match self.component_map.get(&component_id) {
            Some(id) => id,
            None => panic!("No component manager associated with component type {}", type_name::<C>()),
        };
        let manager = match self.component_managers.get(&manager_id) {
            Some(manager) => &**manager,
            None => panic!("Tried to retrieve manager {} with ID {:?} but none exists",
                           type_name::<C>(),
                           manager_id),
        };

        unsafe { restore_manager_trait(manager) }
    }

    pub fn has_manager_for<C: 'static>(&self) -> bool {
        let component_id = ComponentId::of::<C>();
        let manager_id = match self.component_map.get(&component_id) {
            Some(id) => id,
            None => panic!("No component manager associated with component type {}", type_name::<C>()),
        };

        self.component_managers.contains_key(&manager_id)
    }

    pub fn has_manager<T: ComponentManager>(&self) -> bool {
        let manager_id = ManagerId::of::<T>();
        self.component_managers.contains_key(&manager_id)
    }

    /// Reload a component manager for hotloading purposes.
    ///
    /// Clones the component manager from the old scene into the new scene.
    ///
    /// ## Panics
    ///
    /// Panics if the component manager isn't present in the old scene.
    pub fn reload_manager<T: ComponentManager + Clone>(&mut self, old_scene: &Scene) {
        self.register_manager(old_scene.get_manager::<T>().clone());
    }

    /// Reload a component manager or use a default if one was not previously registered.
    ///
    /// Checks the old scene if it has a component manager of the specified type registered. If so
    /// this method behaves identically to `reload_manager()`, otherwise it registers the default
    /// manager with the scene.
    pub fn reload_manager_or_default<T: ComponentManager + Clone>(&mut self, old_scene: &Scene, default: T) {
        if old_scene.has_manager::<T>() {
            self.reload_manager::<T>(old_scene);
        } else {
            self.register_manager(default);
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_manager.borrow().is_alive(entity)
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
        if self.is_alive(entity) {
            for manager in self.component_managers.values() {
                // In this context we don't care what type of component the manager *actually* is
                // for, so we transmute it to `ComponentManager<Component=()>` so that we can
                // tell it to destroy the entity regardless of it's actual type. This is safe to do
                // because the signature of `ComponentManager::destroy()` doesn't change based on
                // the component so we're basically just calling a function pointer.
                let manager = unsafe { restore_manager_trait::<()>(&**manager) };
                manager.destroy(entity);
            }

            let transform_manager = self.get_manager::<TransformManager>();
            transform_manager.walk_children(entity, &mut |entity| {
                for (_, manager) in self.component_managers.iter() {
                    // Same story as above.
                    let manager = unsafe { restore_manager_trait::<()>(&**manager) };
                    manager.destroy(entity);
                }
            });

            self.entity_manager.borrow_mut().destroy(entity);
        }
    }
}

fn type_name<T>() -> &'static str {
    unsafe {
        intrinsics::type_name::<T>()
    }
}

/*
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
*/

unsafe fn restore_manager_trait<'a, T>(trait_object: &'a ComponentManager<Component=()>) -> &'a ComponentManager<Component=T> {
    mem::transmute(trait_object)
}

/// Performs an unchecked downcast from the `ComponentManager` trait object to the concrete type.
unsafe fn downcast_manager<'a, T: ComponentManager, C>(manager: &'a ComponentManager<Component=C>) -> &'a T {
    // Get the raw representation of the trait object.
    let to: TraitObject = mem::transmute(manager);

    // Extract the data pointer.
    mem::transmute(to.data)
}

/// FIXME: WARNING! WARNING! DANGER! VERY BAD!
unsafe fn downcast_manager_mut<'a, T: ComponentManager, C>(manager: &'a ComponentManager<Component=C>) -> &'a mut T {
    // Get the raw representation of the trait object.
    let to: TraitObject = mem::transmute(manager);

    // Extract the data pointer.
    mem::transmute(to.data)
}
