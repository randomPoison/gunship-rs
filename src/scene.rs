use std::collections::HashMap;
use std::cell::RefCell;
use std::intrinsics::type_name;
use std::fmt::{Debug, Error, Formatter};
use std::mem;
use std::ops::{Deref, DerefMut};

use ecs::*;
use engine::*;
use input::Input;

pub struct ManagerMap(HashMap<ManagerId, Box<ComponentManagerBase>>);

impl ManagerMap {
    pub fn new() -> ManagerMap {
        ManagerMap(HashMap::default())
    }
}

impl Clone for ManagerMap {
    fn clone(&self) -> ManagerMap {
        ManagerMap::new()
    }
}

impl Debug for ManagerMap {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // TODO: Actually list all of the manager Ids.
        write!(f, "ManagerMap")
    }
}

impl Deref for ManagerMap {
    type Target = HashMap<ManagerId, Box<ComponentManagerBase>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ManagerMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
#[derive(Debug, Clone)]
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    managers: ManagerMap,
    pub input: Input,
    pub audio_source: ::bs_audio::AudioSource, // FIXME: This is a hot mess, the scene should not have to hold on to the audio source.
}

impl Scene {
    pub fn new(audio_source: ::bs_audio::AudioSource, managers: ManagerMap) -> Scene {
        Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            managers: managers,
            input: Input::new(),
            audio_source: audio_source,
        }
    }

    pub fn get<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.manager_for::<C>().get(entity)
    }

    pub fn get_manager<T: ComponentManager>(&self) -> &T {
        let manager_id = ManagerId::of::<T>();
        let trait_object = match self.managers.get(&manager_id) {
            Some(trait_object) => &**trait_object,
            None => panic!(
                "Tried to retrieve manager {} with ID {:?} but none exists",
                unsafe { type_name::<T>() },
                manager_id),
        };

        unsafe { downcast_ref(trait_object) }
    }

    pub unsafe fn get_manager_mut<T: ComponentManager>(&self) -> &mut T {
        let manager_id = ManagerId::of::<T>();
        let trait_object = match self.managers.get(&manager_id) {
            Some(trait_object) => &**trait_object,
            None => panic!(
                "Tried to retrieve manager {} with ID {:?} but none exists",
                type_name::<T>(),
                manager_id),
        };

        // Use same method as `UnsafeCell` to convert a immutable reference to a mutable pointer.
        downcast_mut(trait_object)
    }

    pub fn manager_for<C: Component>(&self) -> &C::Manager {
        let manager_id = ManagerId::of::<C::Manager>();
        let manager = match self.managers.get(&manager_id) {
            Some(manager) => &**manager, // &Box<()> -> &()
            None => panic!("Tried to retrieve manager {} with ID {:?} but none exists",
                           unsafe { type_name::<C>() },
                           manager_id),
        };

        unsafe { downcast_ref(manager) }
    }

    pub fn has_manager_for<C: Component>(&self) -> bool {
        let manager_id = ManagerId::of::<C::Manager>();

        self.managers.contains_key(&manager_id)
    }

    pub fn has_manager<T: ComponentManager>(&self) -> bool {
        let manager_id = ManagerId::of::<T>();
        self.managers.contains_key(&manager_id)
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let manager = self.manager_for::<T>();
        manager.get(entity)
    }

    pub fn reload_component<T: Component>(&mut self, _old_scene: &Scene) {
        panic!("Hotloading is currently broken, please come back later");
    }

    // /// Reload a component manager for hotloading purposes.
    // ///
    // /// Clones the component manager from the old scene into the new scene.
    // ///
    // /// ## Panics
    // ///
    // /// Panics if the component manager isn't present in the old scene.
    // pub fn reload_manager<T: ComponentManager + Clone>(&mut self, old_scene: &Scene) {
    //     self.register_manager(old_scene.get_manager::<T>().clone());
    // }

    // /// Reload a component manager or use a default if one was not previously registered.
    // ///
    // /// Checks the old scene if it has a component manager of the specified type registered. If so
    // /// this method behaves identically to `reload_manager()`, otherwise it registers the default
    // /// manager with the scene.
    // pub fn reload_manager_or_default<T: ComponentManager + Clone>(&mut self, old_scene: &Scene, default: T) {
    //     if old_scene.has_manager::<T>() {
    //         self.reload_manager::<T>(old_scene);
    //     } else {
    //         self.register_manager(default);
    //     }
    // }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_manager.borrow().is_alive(entity)
    }

    pub fn create_entity(&self) -> Entity {
        self.entity_manager.borrow_mut().create()
    }

    /// Instantiates an instance of the model in the scene, returning the root entity.
    pub fn instantiate_model(&self, resource: &str) -> Entity {
        Engine::resource_manager().instantiate_model(resource, self).unwrap()
    }

    pub fn destroy_entity(&self, entity: Entity) {
        if self.is_alive(entity) {
            // FIXME: Notify the component managers that an entity was asploded.

            // for manager in self.managers.values() {
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
            //     for (_, manager) in self.managers.iter() {
            //         // Same story as above.
            //         manager.destroy(entity);
            //     }
            // });

            self.entity_manager.borrow_mut().destroy(entity);
        }
    }

    // TODO: How do we make this private? I think scene has to be a submodule under engine. We might
    // move `Engine` into the root gunship module which would allow it to access private members
    // everywhere which is helpful.
    pub fn update_managers(&mut self) {
        for (_, manager) in self.managers.iter_mut() {
            manager.update();
        }
    }
}

derive_Singleton!(Scene);

/// Performs an unchecked downcast from `&()` trait object to the concrete type.
unsafe fn downcast_ref<'a, T>(manager: &'a ComponentManagerBase) -> &'a T {
    use std::raw::TraitObject;

    // Get the underlying trait object representation.
    let trait_object: TraitObject = mem::transmute(manager);

    // Cast the data pointer to the correct type and dereference it.
    &*(trait_object.data as *const T)
}

/// Performs an unchecked downcast from `&()` trait object to the concrete type.
unsafe fn downcast_mut<'a, T>(manager: &'a ComponentManagerBase) -> &'a mut T {
    use std::raw::TraitObject;

    // Get the underlying trait object representation.
    let trait_object: TraitObject = mem::transmute(manager);

    // Cast the data pointer to the correct type and dereference it.
    &mut *(trait_object.data as *mut T)
}
