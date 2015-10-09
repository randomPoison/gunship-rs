use std::collections::{HashMap, HashSet};
use std::collections::hash_state::HashState;
use std::cell::{RefCell, Ref};
use std::iter::*;
use std::slice::Iter;

use hash::FnvHashState;
use math::*;
use stopwatch::Stopwatch;

use ecs::*;
use scene::Scene;
use debug_draw;
use super::EntityMap;
use self::grid_collision::GridCollisionSystem;
use self::bounding_volume::bvh_update;
use component::transform::Transform;

pub mod grid_collision;
pub mod bounding_volume;

///! This is the collision sub-system for the game engine. It is composed of two parts: the
///! user-facing `ColliderManager` and a back end collision processing system.
///!
///! In order to enable an entity to be tested for collisions with other entities it must be given
///! a `Collider`. Colliders define basic collision volumes that can be used to perform collision
///! tests. Users can access collider data to configure the collision volumes for their entities.
///!
///! Behind the scenes Gunship can support a number of processing systems to perform the collision
///! detection using the user configured colliders. Maybe the user will have access to the
///! processing system? That would be real useful, but maybe not? It could be useful if the user
///! wants to have more control over the bounding volume hierarchy for each object.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider {
    /// Represents a sphere collider.
    ///
    /// # Details
    ///
    /// The sphere collider is positioned relative entity in world coordinates but is unscaled by
    /// the entity's local or total scale. This is because the sphere collider cannot be deformed
    /// by a non-uniform scale, so the collider must be sized absolutely. If an object changes size
    /// at runtime and the collider needs to match that size users can programatically resize the
    /// object's associated sphere collider.
    Sphere {
        offset: Vector3,
        radius: f32,
    },

    /// Represents a box collider oriented to the entity's local coordinate system.
    Box {
        offset: Vector3,
        width:  Vector3,
    },

    /// Represents a collision geometry derived from mesh data.
    Mesh,
}

/// Manages the user-facing data in the collision system.
#[derive(Debug, Clone)]
pub struct ColliderManager {
    colliders: Vec<RefCell<Collider>>,
    entities:  Vec<Entity>,
    indices:   HashMap<Entity, usize>,

    callback_manager: CollisionCallbackManager,
}

impl ColliderManager {
    pub fn new() -> ColliderManager {
        ColliderManager {
            colliders: Vec::new(),
            entities:  Vec::new(),
            indices:   HashMap::new(),

            callback_manager: CollisionCallbackManager::new(),
        }
    }

    pub fn assign(&mut self, entity: Entity, collider: Collider) {
        debug_assert!(!self.indices.contains_key(&entity));

        let index = self.colliders.len();
        self.colliders.push(RefCell::new(collider));
        self.entities.push(entity);
        self.indices.insert(entity, index);
    }

    pub fn register_callback<T: CollisionCallback + 'static>(&mut self, entity: Entity, callback: T) {
        self.callback_manager.register(entity, callback);
    }

    // TODO: Eeeeeewwwwww, clean this up when abstract return types are added to Rust.
    pub fn iter(&self) -> Zip<Cloned<Iter<Entity>>, Map<Iter<RefCell<Collider>>, fn (&RefCell<Collider>) -> Ref<Collider>>> {
        fn unwrap(refcell_collider: &RefCell<Collider>) -> Ref<Collider> {
            refcell_collider.borrow()
        }

        self.entities.iter()
            .cloned()
            .zip(self.colliders
                .iter()
                .map(unwrap as fn (&RefCell<Collider>) -> Ref<Collider>))
    }
}

impl ComponentManager for ColliderManager {
    fn destroy_all(&self, _entity: Entity) {
        // unimplemented!();
    }

    fn destroy_marked(&mut self) {
        // unimplemented!();
    }
}

/// Combines collider data with calculated world position.
///
/// #Details
///
/// It is common for collision processors to need to reference a collider multiple times in the
/// course of a single processing pass, so it is valueable to only have to retrieve the position
/// data for a collider once and cache off those results.
#[derive(Debug, Clone, Copy)]
pub enum CachedCollider {
    Sphere(Sphere),
    Box(OBB),
    Mesh,
}

impl CachedCollider {
    pub fn from_collider_transform(collider: &Collider, transform: &Transform) -> CachedCollider {
        match collider {
            &Collider::Sphere { offset, radius } => {
                CachedCollider::Sphere(Sphere {
                    center: transform.position_derived() + offset,
                    radius: radius,
                })
            },
            &Collider::Box { offset: _, width: _ } => unimplemented!(),
            &Collider::Mesh => unimplemented!(),
        }
    }

    pub fn debug_draw(&self) {
        match self {
            &CachedCollider::Sphere(Sphere { center, radius }) => {
                debug_draw::sphere(center, radius);
            },
            &CachedCollider::Box(_) => unimplemented!(),
            &CachedCollider::Mesh => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct OBB {
    pub center: Point,
    pub axes: [Vector3; 3],
    pub half_widths: Vector3,
}

#[derive(Debug, Clone)]
pub struct CollisionSystem {
    grid_system: GridCollisionSystem,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {
            grid_system: GridCollisionSystem::new(),
        }
    }
}

impl System for CollisionSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let _stopwatch = Stopwatch::new("collision system");

        bvh_update(scene, delta);
        self.grid_system.update(scene, delta);
        let mut collider_manager = scene.get_manager_mut::<ColliderManager>();
        collider_manager.callback_manager.process_collisions(scene, &self.grid_system.collisions);
    }
}

pub trait CollisionCallback {
    fn invoke(&mut self, scene: &Scene, first: Entity, second: Entity);
}

impl<T: ?Sized + 'static> CollisionCallback for T where T: FnMut(&Scene, Entity, Entity) {
    fn invoke(&mut self, scene: &Scene, first: Entity, second: Entity) {
        self.call_mut((scene, first, second));
    }
}

impl ::std::fmt::Debug for CollisionCallback {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.pad("CollisionCallback")
    }
}

type CallbackId = u64;

fn callback_id<T: CollisionCallback + 'static>() -> CallbackId {
    unsafe { ::std::intrinsics::type_id::<T>() }
}

#[derive(Debug)]
pub struct CollisionCallbackManager {
    callbacks: HashMap<CallbackId, Box<CollisionCallback>, FnvHashState>,
    entity_callbacks: EntityMap<Vec<CallbackId>>,
}

impl CollisionCallbackManager {
    pub fn new() -> CollisionCallbackManager {
        CollisionCallbackManager {
            callbacks: HashMap::default(),
            entity_callbacks: EntityMap::default(),
        }
    }

    pub fn register<T: CollisionCallback + 'static>(&mut self, entity: Entity, callback: T) {
        let callback_id = callback_id::<T>();
        if !self.callbacks.contains_key(&callback_id) {
            self.callbacks.insert(callback_id, Box::new(callback));
        }

        // TODO: Should we allow an entity to be registered with the same callback more than once?
        //       For now I'm going to say no since it seems like that's most likely a logic error.
        if let Some(mut entity_callbacks) = self.entity_callbacks.get_mut(&entity) {
            entity_callbacks.push(callback_id);
            return;
        }

        // TODO: Make this block an else block on the previous if block once non-lexical scopes are
        // added to Rust.
        {
            let entity_callbacks = vec![callback_id];
            self.entity_callbacks.insert(entity, entity_callbacks);
        }
    }

    /// For a pair of colliding entities A and B, we assume that there is either an entry (A, B) or
    /// (B, A), but not both. We manually invoke the callback for both colliding entities.
    pub fn process_collisions<H>(
        &mut self,
        scene: &Scene,
        collisions: &HashSet<(Entity, Entity), H>
    ) where H: HashState {
        let _stopwatch = Stopwatch::new("process collision callbacks");

        for pair in collisions {
            if let Some(callback_ids) = self.entity_callbacks.get(&pair.0) {
                for callback_id in callback_ids.iter() {
                    let mut callback = self.callbacks.get_mut(callback_id).unwrap();
                    callback.invoke(scene, pair.0, pair.1);
                }
            }

            if let Some(callback_ids) = self.entity_callbacks.get(&pair.1) {
                for callback_id in callback_ids.iter() {
                    let mut callback = self.callbacks.get_mut(callback_id).unwrap();
                    callback.invoke(scene, pair.1, pair.0);
                }
            }
        }
    }
}

impl Clone for CollisionCallbackManager {
    // TODO: Handle re-registering callbacks when cloning.
    fn clone(&self) -> CollisionCallbackManager {
        CollisionCallbackManager {
            callbacks: HashMap::default(),
            entity_callbacks: self.entity_callbacks.clone(),
        }
    }
}
