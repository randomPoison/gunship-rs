use std::collections::HashMap;
use std::cell::RefCell;
use std::intrinsics::type_name;
use std::mem;

use ecs::*;
use engine::*;
use input::Input;

/// Contains all the data that defines the current state of the world.
///
/// This is passed into systems in System::update(). It can be used access component
/// managers and input.
#[derive(Debug, Clone)]
pub struct Scene {
    entity_manager: RefCell<EntityManager>,
    component_managers: HashMap<ManagerId, Box<()>>,
    pub input: Input,
    pub audio_source: ::bs_audio::AudioSource, // FIXME: This is a hot mess, the scene should not have to hold on to the audio source.
}

impl Scene {
    pub fn new(audio_source: ::bs_audio::AudioSource) -> Scene {
        Scene {
            entity_manager: RefCell::new(EntityManager::new()),
            component_managers: HashMap::new(),
            input: Input::new(),
            audio_source: audio_source,
        }
    }

    pub fn get_manager<T: ComponentManager>(&self) -> &T {
        let manager_id = ManagerId::of::<T>();
        let trait_object = match self.component_managers.get(&manager_id) {
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
        let trait_object = match self.component_managers.get(&manager_id) {
            Some(trait_object) => &**trait_object,
            None => panic!(
                "Tried to retrieve manager {} with ID {:?} but none exists",
                type_name::<T>(),
                manager_id),
        };

        // Use same method as `UnsafeCell` to convert a immutable reference to a mutable pointer.
        downcast_mut(&mut *(trait_object as *const () as *mut ()))
    }

    pub fn get_manager_for<C: Component>(&self) -> &C::Manager {
        let manager_id = ManagerId::of::<C::Manager>();
        let manager = match self.component_managers.get(&manager_id) {
            Some(manager) => &**manager, // &Box<()> -> &()
            None => panic!("Tried to retrieve manager {} with ID {:?} but none exists",
                           unsafe { type_name::<C>() },
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

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let manager = self.get_manager_for::<T>();
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
