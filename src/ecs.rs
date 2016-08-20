use collections::EntitySet;
use engine::{Engine, EngineBuilder};
use scene::Scene;
use std::collections::VecDeque;
use std::intrinsics;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity(u32);

impl Entity {
    pub fn new() -> Entity {
        Engine::scene().create_entity()
    }
}

const MIN_RECYCLED_ENTITIES: usize = 1000;

#[derive(Debug, Clone)]
pub struct EntityManager {
    entities: EntitySet,
    recycled_entities: VecDeque<Entity>,
    marked_for_destroy: Vec<Entity>,
    id_counter: u32
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            entities: EntitySet::default(),
            recycled_entities: VecDeque::new(),
            marked_for_destroy: Vec::new(),
            id_counter: 1
        }
    }

    pub fn create(&mut self) -> Entity {
        if self.recycled_entities.len() > MIN_RECYCLED_ENTITIES {
            self.recycled_entities.pop_front().unwrap()
        } else {
            let entity = Entity(self.id_counter);
            self.id_counter += 1;
            self.entities.insert(entity);
            entity
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        let removed = self.entities.remove(&entity);
        debug_assert!(removed, "Trying to destroy entity {:?} but it is not live");

        self.recycled_entities.push_back(entity);
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
}

pub trait System: 'static {
    fn update(&mut self, scene: &Scene, delta: f32);
}

impl<T: ?Sized> System for T
    where T: 'static + FnMut(&Scene, f32) {
    fn update(&mut self, scene: &Scene, delta: f32) {
        self.call_mut((scene, delta));
    }
}

pub trait ComponentManagerBase: 'static {
    fn update(&mut self) {}
}

pub trait ComponentManager: ComponentManagerBase + Sized {
    type Component: Component<Manager=Self>;

    fn register(builder: &mut EngineBuilder);
    // fn apply(&mut self, entity: Entity, message: <Self::Component as Component>::Message);
    fn get(&self, entity: Entity) -> Option<&Self::Component>;
    fn destroy(&self, entity: Entity);
}

pub trait Component: 'static + Clone {
    type Manager: ComponentManager<Component=Self>;
    type Message;
}

/// Trait for defining behavior that is associated with a specific component.
pub trait ComponentUpdate {
    fn update(&mut self);
}

/// Helper trait used to allow generic component managers like `DefaultManager` and
/// `SingletonManager` to support custom message types.
pub trait Message: 'static + Sized {
    type Target: Component<Message=Self>;

    fn apply(self, component: &mut Self::Target);
}

// ===============
// CUSTOM TYPE IDS
// ===============

#[cfg(not(feature = "hotloading"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManagerId(u64);

#[cfg(feature = "hotloading")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManagerId(&'static str);

impl ManagerId {
    #[cfg(not(feature = "hotloading"))]
    pub fn of<T: ComponentManager>() -> ManagerId {
        unsafe { ManagerId(intrinsics::type_id::<T>()) }
    }

    /// Two cases:
    ///
    /// - No template (e.g. `foo::bar::TransformManager`) just remove path (becomes `TransformManager`).
    /// - Template (e.g. `foo::bar::Manager<foo::bar::Foo>`) innermost type without leading path
    ///   (becomes `Foo`).
    #[cfg(feature = "hotloading")]
    pub fn of<T: ComponentManager>() -> ManagerId {
        let full_name = unsafe { intrinsics::type_name::<T>() };

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
pub struct SystemId(u64);

#[cfg(feature = "hotloading")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemId(&'static str);

impl SystemId {
    #[cfg(not(feature = "hotloading"))]
    pub fn of<T: System>() -> SystemId {
        unsafe { SystemId(intrinsics::type_id::<T>()) }
    }

    /// Two cases:
    ///
    /// - No template (e.g. `foo::bar::TransformManager`) just remove path (becomes `TransformManager`).
    /// - Template (e.g. `foo::bar::Manager<foo::bar::Foo>`) innermost type without leading path
    ///   (becomes `Foo`).
    #[cfg(feature = "hotloading")]
    pub fn of<T: System>() -> SystemId {
        let full_name = unsafe {
            intrinsics::type_name::<T>()
        };

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

        SystemId(&sub_str[slice_index..])
    }
}
