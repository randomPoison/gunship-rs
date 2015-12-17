use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::intrinsics;
use std::mem;

use ecs::*;
use input::Input;
use component::{Transform, Camera, Light, Mesh, AudioSource,
                AlarmId, Collider};
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

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    component_managers: HashMap<ManagerId, Box<()>>,
    pub input: Input,
    pub audio_source: ::bs_audio::AudioSource, // FIXME: This is a hot mess, the scene should not have to hold on to the audio source.
    resource_manager: Rc<ResourceManager>,
}

impl Scene {
    pub fn new(resource_manager: &Rc<ResourceManager>, audio_source: ::bs_audio::AudioSource) -> Scene {
        let mut scene = Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            component_managers: HashMap::new(),
            input: Input::new(),
            audio_source: audio_source,
            resource_manager: resource_manager.clone(),
        };

        // Register internal component managers.
        scene.register_component::<Transform>();
        scene.register_component::<Camera>();
        scene.register_component::<Light>();
        scene.register_component::<Mesh>();
        scene.register_component::<AudioSource>();
        scene.register_component::<AlarmId>();
        scene.register_component::<Collider>();

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
        scene.reload_component::<Transform>(self);
        scene.reload_component::<Camera>(self);
        scene.reload_component::<Light>(self);
        scene.reload_component::<Mesh>(self);
        scene.reload_component::<AudioSource>(self);
        scene.reload_component::<AlarmId>(self);
        scene.reload_component::<Collider>(self);

        scene
    }

    pub fn register_component<C: Component>(&mut self) {
        C::Manager::register(self);
    }

    pub fn register_manager<T: ComponentManager>(&mut self, manager: T) {
        let manager_id = ManagerId::of::<T>();
        assert!(
            !self.component_managers.contains_key(&manager_id),
            "Manager {} with ID {:?} already registered", type_name::<T>(), manager_id);

        // Box the manager as a trait object to construct the data and vtable pointers.
        let boxed_manager = Box::new(manager);

        // Transmute to raw trait object to throw away type information so that we can store it
        // in the type map.
        let boxed_trait_object = unsafe { mem::transmute(boxed_manager) };

        // Add the manager to the type map and the component id to the component map.
        self.component_managers.insert(manager_id.clone(), boxed_trait_object);
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

        unsafe { downcast_ref(trait_object) }
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

        // Use same method as `UnsafeCell` to convert a immutable reference to a mutable pointer.
        unsafe { downcast_mut(&mut *(trait_object as *const () as *mut ())) }
    }

    pub fn get_manager_for<C: Component>(&self) -> &C::Manager {
        let manager_id = ManagerId::of::<C::Manager>();
        let manager = match self.component_managers.get(&manager_id) {
            Some(manager) => &**manager, // &Box<()> -> &()
            None => panic!("Tried to retrieve manager {} with ID {:?} but none exists",
                           type_name::<C>(),
                           manager_id),
        };

        unsafe { downcast_ref(manager) }
    }

    pub fn has_manager_for<C: Component>(&self) -> bool {
        let manager_id = ManagerId::of::<C::Manager>();

        self.component_managers.contains_key(&manager_id)
    }

    pub fn has_manager<T: ComponentManager>(&self) -> bool {
        let manager_id = ManagerId::of::<T>();
        self.component_managers.contains_key(&manager_id)
    }

    pub fn reload_component<T: Component>(&mut self, _old_scene: &Scene) {
        panic!("Hotloading is currently broken, please come back later");
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

    pub fn resource_manager(&self) -> Rc<ResourceManager> {
        self.resource_manager.clone()
    }

    /// Instantiates an instance of the model in the scene, returning the root entity.
    pub fn instantiate_model(&self, resource: &str) -> Entity {
        self.resource_manager.clone().instantiate_model(resource, self).unwrap()
    }

    pub fn destroy_entity(&self, entity: Entity) {
        if self.is_alive(entity) {
            // FIXME: Notify the component managers that an entity was asploded.

            // for manager in self.component_managers.values() {
            //     // In this context we don't care what type of component the manager *actually* is
            //     // for, so we transmute it to `ComponentManager<Component=()>` so that we can
            //     // tell it to destroy the entity regardless of it's actual type. This is safe to do
            //     // because the signature of `ComponentManager::destroy()` doesn't change based on
            //     // the component so we're basically just calling a function pointer.
            //     manager.destroy(entity);
            // }
            //
            // let transform_manager = self.get_manager::<TransformManager>();
            // transform_manager.walk_children(entity, &mut |entity| {
            //     for (_, manager) in self.component_managers.iter() {
            //         // Same story as above.
            //         manager.destroy(entity);
            //     }
            // });

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

/// Performs an unchecked downcast from `&()` trait object to the concrete type.
unsafe fn downcast_ref<T>(manager: &()) -> &T {
    // We're just transmuting a pointer to `()` to a pointer to `T`.
    mem::transmute(manager)
}

/// Performs an unchecked downcast from `&()` trait object to the concrete type.
unsafe fn downcast_mut<T>(manager: &mut ()) -> &mut T {
    // We're just transmuting a pointer to `()` to a pointer to `T`.
    mem::transmute(manager)
}
