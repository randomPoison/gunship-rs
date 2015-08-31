use std::collections::HashMap;
use std::cell::{RefCell, Ref};
use std::iter::*;
use std::slice::Iter;

use math::*;
use ecs::*;
use scene::Scene;
use self::grid_collision::GridCollisionSystem;
use self::bounding_volume::BoundingVolumeUpdateSystem;

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
}

impl ColliderManager {
    pub fn new() -> ColliderManager {
        ColliderManager {
            colliders: Vec::new(),
            entities:  Vec::new(),
            indices:   HashMap::new(),
        }
    }

    pub fn assign(&mut self, entity: Entity, collider: Collider) {
        debug_assert!(!self.indices.contains_key(&entity));

        let index = self.colliders.len();
        self.colliders.push(RefCell::new(collider));
        self.entities.push(entity);
        self.indices.insert(entity, index);
    }

    pub fn iter(&self) -> Zip<Cloned<Iter<Entity>>, Map<Iter<RefCell<Collider>>, fn (&RefCell<Collider>) -> Ref<Collider>>> {
            fn unwrap(refcell_collider: &RefCell<Collider>) -> Ref<Collider> {
            refcell_collider.borrow()
        }

        self.entities.iter().cloned().zip(self.colliders.iter().map(unwrap as fn (&RefCell<Collider>) -> Ref<Collider>))
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
pub struct CachedCollider {
    position: Point,
    orientation: Quaternion,
    scale: Vector3,
    collider: Collider,
    entity: Entity,
}

#[derive(Debug, Clone)]
pub struct CollisionSystem {
    grid_system: GridCollisionSystem,
    bvh_system:  BoundingVolumeUpdateSystem,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {
            grid_system: GridCollisionSystem::new(),
            bvh_system: BoundingVolumeUpdateSystem,
        }
    }
}

impl System for CollisionSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        self.bvh_system.update(scene, delta);
        self.grid_system.update(scene, delta);
    }
}
