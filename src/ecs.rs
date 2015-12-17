use collections::EntitySet;
use component::DefaultManager;
use std::collections::VecDeque;

use scene::Scene;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity(u32);

impl Entity {
    /// Retrieves the component `T` associated with this entity if one exists.
    pub fn get_component_mut<'a: 'b, 'b, T: 'static>(&self, _: &Scene) -> Option<&'b mut T> {
        // let manager = scene.get_manager_for::<T>();
        // manager.get(*self) // TODO: Make this happen
        None
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

pub trait System {
    fn update(&mut self, scene: &Scene, delta: f32);
}

impl<T: ?Sized> System for T where T: FnMut(&Scene, f32) {
    fn update(&mut self, scene: &Scene, delta: f32) {
        self.call_mut((scene, delta));
    }
}

pub trait ComponentManager: 'static + Sized {
    type Component: Component<Manager=Self>;

    fn register(scene: &mut Scene);
    fn destroy(&self, entity: Entity);
}

pub trait Component: 'static + Clone {
    type Manager: ComponentManager<Component=Self> = DefaultManager<Self>;
}

impl Component for () {}
