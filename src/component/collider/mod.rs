//! The collision subsystem for the engine, including the user-facing API and the backend collision
//! processing systems.
//!
//! The collision system is composed of two parts: the user-facing `ColliderManager` and a back end
//! collision processing system. The system is designed to support different back ends while
//! maintaining a consistent API for game code. This should allow for game developers to use a
//! collision processing system that is best suited for their particular game while still allowing
//! experimentation and rapid iteration (i.e. developers can start with the default collision
//! engine and then switch to a more suitable one without having to change any of their code).
//!
//! User Facing API
//! ===============
//!
//! In order to enable an entity to be tested for collisions with other entities it must be given
//! a `Collider`. Colliders define basic collision volumes that can be used to perform collision
//! tests. Colliders are assigned to entities and can be retrieved through the `ColliderManager`:
//!
//! ```rust
//! fn update(scene: &Scene) {
//!     // Retrieve the collider manager from the scene.
//!     let collider_manager = scene.get_manager::<ColliderManager>();
//!
//!     // Create an entity and assign it a sphere collider.
//!     // N.B. Entites also need to be assigned a `Transform` component for the Collider to work.
//!     let entity = scene.create_entity();
//!     collider_manager.assign(entity, Collider::Sphere {
//!         offset: Vector3::zero(),
//!         radius: 1.0,
//!     });
//! }
//! ```
//!
//! There are currently two types of collision primitives supported (oriented boxes and spheres)
//! with several more types planned for support in the future (meshes, axis-aligned boxes,
//! capsules, and planes). Each of these types have configurable options, such as the radius for
//! sphere colliders, and all colliders are placed in the world based on the entity's transform.
//! Each collider has a natural "anchor point", which is by default located at the transform's
//! world position (e.g. the sphere and box colliders' anchor are their center point), and all
//! collider types have an `offset` member that allows for the collider to be shifted relative to
//! the entity's transform.
//!
//! Collision Callbacks
//! -------------------
//!
//! In order to be notified when collisions are detected game code must register a callback with
//! the collision system. The collision callback can be any `CollisionCallback` trait object or
//! function that has the same signature as `CollisionCallback::invoke()` (including closures).
//! A callback is associated with one or more entities and will be invoked every frame where at
//! least one of its entities are detected as part of a collision. A collision callback will be
//! invoked at most once per frame -- they are intended to behave like system updates where
//! high-overhead operations (e.g. retrieving component managers) is done once and then all
//! relevant entities are processed at once.
//!
//! Multiple callbacks can be associated with the same entity so that callbacks don't have to
//! perform multiple functions and different behaviors can be mixed and matched for different
//! entities. The recommended way to use collision callbacks is perform collision behavior for a
//! "class" of game entities without having to perform additional runtime checks. For example: If
//! your game is about ducks eating breadcrumbs and you want you duck entity to eat a breadcrumb
//! when it collides with the breadcrumb, you might have something as follows:
//!
//! ```rust
//! fn duck_callback(scene: &Scene, collisions: &[Collision]) {
//!     let duck_manager = scene.get_manager::<DuckManager>();
//!     let bread_manager = scene.get_manager::<BreadManager>();
//!
//!     // Iterate over the collisions.
//!     for collision in collisions {
//!
//!         // Ducks could collide with things other than breadcrubms,
//!         // so check if we've actually collided with breadcrumb.
//!         if bread_manager.has_component(collision.other) {
//!
//!             // Retrieve the `Duck` object from the duck manager.
//!             let mut duck = duck_manager.get_mut(collision.entity).unwrap();
//!
//!             // Give the duck the breadcrumb.
//!             duck.gain_breadcrumb();
//!
//!             // Destroy the breadcrumb entity now that the duck has eaten it.
//!             scene.destroy(collision.other);
//!         }
//!     }
//! }
//! ```
//!
//! Notice how this code never checks that the colliding entities have a `Duck` component, it can
//! safely unwrap the `Option<Duck>` that `get_mut()` returns because `duck_callback()` has only
//! been registered with with entities that have a duck component. There are only two ways for the
//! unwrap to panic: If the duck component has since been removed from the entity or if the Entity
//! never had the component in the first place. In either case the unwrap failing indicates a bug
//! in game code (either assigning the callback without the component or removing the component
//! without unasigning the callback).
//!
//! Back End System
//! ===============
//!
//! The back end system is responsible for performing all of the collision processing every frame.
//! Collision procesing is broken into 4 steps:
//!
//! 1. Map user-defined colliders into usable collision volumes. This includes mapping the volumes
//!    from local space to world space and apply appropriate transformations such as scale and
//!    rotation.
//! 2. Broadphase collision processing. This step culls impossible collision pairs and builds a
//!    list of potential collision pairs to be tested further. This phase is meant to be very
//!    coarse and helps keep collision processing fast by minimizing the number of expensive
//!    collision tests that are being performed through cheaper preprocessing. This phase is also
//!    the most configurable since different forms of broadphase processing work better for
//!    different kinds of games, whereas narrowphase processing can't be substantially changed or
//!    optimized.
//! 3. Narrowphase collision processing. This step takes the list of potential collision pairs
//!    and performs the final collision test to determine which pairs of collision volumes are
//!    intersecting. This step also gathers any other collision information which may be forwarded
//!    to game code such as a list of collision points.
//! 4. Collision callbacks. This step takes the list of detected collisions from narrowphase and
//!    invokes the appropriate collision callbacks with the lists of colliding entities.
//!
//! Currently the only back end system supported is the grid collision system. For more details
//! see the `grid_collision` module below.

use callback::*;
use collections::{EntityMap, EntitySet};
use component::transform::Transform;
use debug_draw;
use ecs::*;
use engine::*;
use math::*;
use scene::Scene;
use self::bounding_volume::{BoundingVolumeManager, bvh_update};
use self::grid_collision::GridCollisionSystem;
use std::cell::{RefCell, Ref, RefMut};
use stopwatch::Stopwatch;
use super::DefaultMessage;
use super::struct_component_manager::{StructComponentManager, Iter};

pub mod grid_collision;
pub mod bounding_volume;

/// An enum representing all possible collision volumes. See each variant for more information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider {
    /// Represents a sphere collider.
    ///
    /// Details
    /// =======
    ///
    /// The sphere collider is positioned at its entity's global position anchored at its center.
    /// If the offset is `<0, 0, 0>` then the sphere's center will be at the transform's position,
    /// otherwise the sphere's center will be positioned at `transform.position_derived()
    /// + sphere.offset`. Sphere colliders are unscaled by the entity's local or total scale. This
    /// is because the sphere collider cannot be deformed by a non-uniform scale, so the collider
    /// must be sized absolutely. If an entity's size changes at runtime and the collider needs to
    /// match that size users can programatically resize the object's associated sphere collider.
    Sphere {
        /// The offset between the sphere's center and the entity's global position.
        offset: Vector3,

        /// The radius in global units of the sphere collider. This is unaffected by the
        /// transform's scale.
        radius: f32,
    },

    /// Represents a box collider oriented to the entity's local coordinate system.
    ///
    /// Details
    /// =======
    ///
    /// The box collider is positioned at its entity's global position anchored at its center.
    /// if the offset is `<0, 0, 0>` then the box's center will be that the transforms position,
    /// otherwise the box's center will be positioned at `transform.position_derived() + box.offset`.
    /// Box colliders are affected by both the transform's scale and orientation. The box is
    /// oriented according to the transform's derived orientation, and scaled according it its
    /// local scale. The `widths` member defines the base width along each axis, which are
    /// multiplied by the corresponding axis of the transform's local scale. For example: If you
    /// have a box collider with widths `<1, 2, 1>` and has a local scale of `<1, 2, 3>` the box
    /// collider will be processed by the collision system as having widths `<1, 4, 3>`.
    Box {
        offset: Vector3,
        widths:  Vector3,
    },

    /// Placeholder for mesh collider. Mesh colliders aren't supported currently, but they will be!
    Mesh,
}

impl Component for Collider {
    type Manager = ColliderManager;
    type Message = DefaultMessage<Collider>;
}

/// Manages the user-facing data in the collision system.
///
/// `ColliderManager` is used to assign a collision volume to entities, as well as retrieve
/// existing colliders. It can be retrieved from the scene with `Scene::get_manager()` and
/// `Scene::get_manager_mut()`. It is also used to register collision callbacks and associate them
/// with entities.
#[derive(Debug, Clone)]
pub struct ColliderManager {
    inner: StructComponentManager<Collider>,
    callback_manager: RefCell<CollisionCallbackManager>,
    bvh_manager: RefCell<BoundingVolumeManager>,
    marked_for_destroy: RefCell<EntitySet>,
}

impl ColliderManager {
    /// Construct a new `ColliderManager`.
    pub fn new() -> ColliderManager {
        ColliderManager {
            inner: StructComponentManager::new(),
            callback_manager: RefCell::new(CollisionCallbackManager::new()),
            bvh_manager: RefCell::new(BoundingVolumeManager::new()),
            marked_for_destroy: RefCell::new(EntitySet::default()),
        }
    }

    /// Assign a collider to the specified entity.
    ///
    /// Panics
    /// ======
    ///
    /// Panics if the specified entity already has a collider component.
    pub fn assign(&self, entity: Entity, collider: Collider) {
        self.inner.assign(entity, collider);
    }

    /// Registers a collision callback without associating it with an entity.
    ///
    /// Details
    /// =======
    ///
    /// This method exists for hotloading support. Callbacks cannot be automatically reloaded when
    /// hotloading occurs so game code must manually re-register callbacks in the post-hotload
    /// setup. The callback manager retains the association between entities and their callbacks
    /// even after hotloading so registering a callback is only needed in order for the collision
    /// system to be able to invoke that callback after hotloading. When associating a callback
    /// with an entity `register_callback()` does not need to be called before `assign_callback()`
    /// as `assign_callback()` will automatically handle registering a new callback.
    pub fn register_callback<T: CollisionCallback + 'static>(&self, callback: T) {
        self.callback_manager.borrow_mut().register(callback);
    }

    /// Assigns a callback to the specified entity.
    ///
    /// Details
    /// =======
    ///
    /// Internally the callback manager (which is owned by the collider manager) keeps a list of
    /// which callbacks are assigned to which entities. After collision processing has happened the
    /// callback manager uses that information to build a list of all of the collisions that should
    /// be passed when invoking that callback.
    ///
    /// For more information see the module documentation.
    pub fn assign_callback<T: CollisionCallback + 'static>(&self, entity: Entity, callback: T) {
        self.callback_manager.borrow_mut().assign(entity, callback);
    }

    pub fn iter(&self) -> Iter<Collider> {
        self.inner.iter()
    }

    pub fn bvh_manager(&self) -> Ref<BoundingVolumeManager> {
        self.bvh_manager.borrow()
    }

    pub fn bvh_manager_mut(&self) -> RefMut<BoundingVolumeManager> {
        self.bvh_manager.borrow_mut()
    }
}

impl ComponentManagerBase for ColliderManager {}

impl ComponentManager for ColliderManager {
    type Component = Collider;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(ColliderManager::new());
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.inner.get(entity)
    }

    fn destroy(&self, entity: Entity) {
        self.inner.destroy(entity);
        self.marked_for_destroy.borrow_mut().insert(entity);
    }
}

/// Combines collider data with calculated world position.
///
/// Details
/// =======
///
/// It is common for collision processors to need to reference a collider multiple times in the
/// course of a single processing pass, so it is valueable to only have to retrieve the position
/// data for a collider once and cache off those results. Before broadphase is run the collision
/// processing system creates a `CachedCollider` from each registered `Collider` component. The
/// collision processing system is then able to operatie on the colliders without needing to access
/// andy other data or component managers.
///
/// Like `Collider` this is an enum, but whereas `Collider` uses struct variants, `CachedCollider`
/// opts to create separate types for each variant. This allows for each variant to have it's own
/// member functions which helps to clean up the collision testing code a bit.
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
            &Collider::Box { offset, widths } => {
                let half_widths = widths * transform.scale_derived() * 0.5;
                let center = transform.position_derived() + offset;
                let orientation = Matrix3::from_quaternion(transform.rotation_derived());

                let obb = OBB {
                    center: center,
                    orientation: orientation,
                    half_widths: half_widths,
                };
                CachedCollider::Box(obb)
            },
            &Collider::Mesh => unimplemented!(),
        }
    }

    pub fn test(&self, other: &CachedCollider) -> bool {
        match self {
            &CachedCollider::Sphere(sphere) => {
                sphere.test_collider(other)
            },
            &CachedCollider::Box(obb) => {
                obb.test_collider(other)
            },
            &CachedCollider::Mesh => unimplemented!(),
        }
    }

    pub fn debug_draw(&self) {
        self.debug_draw_color(color::WHITE);
    }

    pub fn debug_draw_color(&self, color: Color) {
        match self {
            &CachedCollider::Sphere(Sphere { center, radius }) => {
                debug_draw::sphere_color(center, radius, color);
            },
            &CachedCollider::Box(obb) => {
                let transform =
                    Matrix4::from_point(obb.center)
                  * Matrix4::from_matrix3(obb.orientation)
                  * Matrix4::from_scale_vector(obb.half_widths * 2.0);
                debug_draw::box_matrix_color(transform, color);
            },
            &CachedCollider::Mesh => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
}

impl Sphere {
    pub fn test_collider(&self, other: &CachedCollider) -> bool {
        match *other {
            CachedCollider::Sphere(sphere) => self.test_sphere(&sphere),
            CachedCollider::Box(obb) => self.test_obb(&obb),
            CachedCollider::Mesh => unimplemented!(),
        }
    }

    pub fn test_sphere(&self, other: &Sphere) -> bool {
        let dist_sqr = (self.center - other.center).magnitude_squared();
        let max_dist_sqr = (self.radius + other.radius) * (self.radius + other.radius);
        let diff = dist_sqr - max_dist_sqr;

        diff < 0.0 || diff.is_zero()
    }

    pub fn test_obb(&self, obb: &OBB) -> bool {
        let dist_sqr = obb.closest_distance_squared(self.center);
        dist_sqr < self.radius * self.radius
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OBB {
    pub center: Point,
    pub orientation: Matrix3,
    pub half_widths: Vector3,
}

impl OBB {
    pub fn test_collider(&self, other: &CachedCollider) -> bool {
        match other {
            &CachedCollider::Sphere(sphere) => sphere.test_obb(self),
            &CachedCollider::Box(ref obb) => self.test_obb(obb),
            &CachedCollider::Mesh => unimplemented!(),
        }
    }

    pub fn test_obb(&self, b: &OBB) -> bool {
        // Compute rotation matrix expressing b in a's coordinate frame.
        let r = {
            let mut r: Matrix3 = unsafe { ::std::mem::uninitialized() };
            for row in 0..3 {
                for col in 0..3 {
                    r[row][col] = self.orientation.col(row).dot(b.orientation.col(col));
                }
            }
            r
        };

        // Compute translation vector `t`.
        let t = b.center - self.center;

        // Bring translation into a's coordinate frame.
        let t = t * self.orientation.transpose();

        // Compute common subexpressions. Add in an epsilon term to counteract arithmetic errors
        // when two edges are parallel and their cross product is (near) null.
        let abs_r = {
            let mut abs_r: Matrix3 = unsafe { ::std::mem::uninitialized() };
            for row in 0..3 {
                for col in 0..3 {
                    abs_r[row][col] = r[row][col].abs() + EPSILON;
                }
            }
            abs_r
        };

        // Test axes L = A0, L = A1, L = A2.
        for i in 0..3 {
            let ra = self.half_widths[i];
            let rb = b.half_widths.dot(abs_r[i]);

            if t[i].abs() > ra + rb {
                return false;
            }
        }

        // Test axes L = B0, L = B1, L = B2.
        for i in 0..3 {
            let ra = self.half_widths.dot(abs_r.col(i));
            let rb = b.half_widths[i];

            if t.dot(r.col(i)).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A0 x B0.
        {
            let ra = self.half_widths[1] * abs_r[2][0] + self.half_widths[2] * abs_r[1][0];
            let rb =    b.half_widths[1] * abs_r[0][2] +    b.half_widths[2] * abs_r[0][1];
            if (t[2] * r[1][0] - t[1] * r[2][0]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A0 x B1.
        {
            let ra = self.half_widths[1] * abs_r[2][1] + self.half_widths[2] * abs_r[1][1];
            let rb =    b.half_widths[0] * abs_r[0][2] +    b.half_widths[2] * abs_r[0][0];
            if (t[2] * r[1][1] - t[1] * r[2][1]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A0 x B2.
        {
            let ra = self.half_widths[1] * abs_r[2][2] + self.half_widths[2] * abs_r[1][2];
            let rb =    b.half_widths[0] * abs_r[0][1] +    b.half_widths[1] * abs_r[0][0];
            if (t[2] * r[1][2] - t[1] * r[2][2]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A1 x B0.
        {
            let ra = self.half_widths[0] * abs_r[2][0] + self.half_widths[2] * abs_r[0][0];
            let rb =    b.half_widths[1] * abs_r[1][2] +    b.half_widths[2] * abs_r[1][1];
            if (t[0] * r[2][0] - t[2] * r[0][0]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A1 x B1.
        {
            let ra = self.half_widths[0] * abs_r[2][1] + self.half_widths[2] * abs_r[0][1];
            let rb =    b.half_widths[0] * abs_r[1][2] +    b.half_widths[2] * abs_r[1][0];
            if (t[0] * r[2][1] - t[2] * r[0][1]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A1 x B2.
        {
            let ra = self.half_widths[0] * abs_r[2][2] + self.half_widths[2] * abs_r[0][2];
            let rb =    b.half_widths[0] * abs_r[1][1] +    b.half_widths[1] * abs_r[1][0];
            if (t[0] * r[2][2] - t[2] * r[0][2]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A2 x B0.
        {
            let ra = self.half_widths[0] * abs_r[1][0] + self.half_widths[1] * abs_r[0][0];
            let rb =    b.half_widths[1] * abs_r[2][2] +    b.half_widths[2] * abs_r[2][1];
            if (t[1] * r[0][0] - t[0] * r[1][0]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A2 x B1.
        {
            let ra = self.half_widths[0] * abs_r[1][1] + self.half_widths[1] * abs_r[0][1];
            let rb =    b.half_widths[0] * abs_r[2][2] +    b.half_widths[2] * abs_r[2][0];
            if (t[1] * r[0][1] - t[0] * r[1][1]).abs() > ra + rb {
                return false;
            }
        }

        // Test axis L = A2 x B2.
        {
            let ra = self.half_widths[0] * abs_r[1][2] + self.half_widths[1] * abs_r[0][2];
            let rb =    b.half_widths[0] * abs_r[2][1] +    b.half_widths[1] * abs_r[2][0];
            if (t[1] * r[0][2] - t[0] * r[1][2]).abs() > ra + rb {
                return false;
            }
        }

        // Since no separating axis found, the OBBs must be intersecting.
        true
    }

    /// Calculates the closest point to the given point on (or in) the OBB.
    pub fn closest_point(&self, point: Point) -> Point {
        let d = point - self.center;

        // Start result at center of the box, make steps from there.
        let mut result = self.center;
        for axis in 0..3 {
            let local_axis = self.orientation.col(axis);

            // Project d onto the axis to get the distance along the axis of d from the box center.
            let dist = d.dot(local_axis);

            // If the distance is further than the box's extents clamp to the box.
            let dist = dist.clamp(-self.half_widths[axis], self.half_widths[axis]);

            // Step that distance along the axis to get the world coordinate.
            result += dist * local_axis;
        }

        result
    }

    /// Calculates the squared distance between the given point and the OBB.
    pub fn closest_distance_squared(&self, point: Point) -> f32 {
        let closest = self.closest_point(point);
        (point - closest).magnitude_squared()
    }
}

#[derive(Clone)]
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

        let collider_manager = scene.get_manager::<ColliderManager>();
        {
            let bvh_manager = collider_manager.bvh_manager_mut();
            self.grid_system.update(&*bvh_manager);
        }

        collider_manager.callback_manager.borrow_mut().process_collisions(scene, &self.grid_system.collisions);

        // Run cleanup of marked components.
        let mut marked_for_destroy = collider_manager.marked_for_destroy.borrow_mut();
        for entity in marked_for_destroy.drain() {
            collider_manager.callback_manager.borrow_mut().unregister_all(entity);
            collider_manager.bvh_manager.borrow_mut().destroy_immediate(entity);
        }
    }
}

pub trait CollisionCallback {
    fn invoke(&mut self, scene: &Scene, first: Entity, others: &[Entity]);
}

impl<T: ?Sized + 'static> CollisionCallback for T where T: FnMut(&Scene, Entity, &[Entity]) {
    fn invoke(&mut self, scene: &Scene, first: Entity, others: &[Entity]) {
        self.call_mut((scene, first, others));
    }
}

impl ::std::fmt::Debug for CollisionCallback {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.pad("CollisionCallback")
    }
}

#[derive(Debug, Clone)]
pub struct CollisionCallbackManager {
    callbacks: CallbackManager<CollisionCallback>,
    entity_callbacks: EntityMap<Vec<CallbackId>>,
    entity_collisions: EntityMap<Vec<Entity>>,
}

impl CollisionCallbackManager {
    pub fn new() -> CollisionCallbackManager {
        CollisionCallbackManager {
            callbacks: CallbackManager::new(),
            entity_callbacks: EntityMap::default(),
            entity_collisions: EntityMap::default(),
        }
    }

    fn register<T: 'static + CollisionCallback>(&mut self, callback: T) {
        let callback_id = CallbackId::of::<T>();
        self.callbacks.register(callback_id, Box::new(callback));
    }

    #[allow(unused_variables)]
    fn assign<T: 'static + CollisionCallback>(&mut self, entity: Entity, callback: T) {
        let callback_id = CallbackId::of::<T>();
        debug_assert!(
            self.callbacks.get(&callback_id).is_some(),
            "Cannot assign collision callback that has not been registered");

        // TODO: Should we allow an entity to be registered with the same callback more than once?
        //       For now I'm going to say no since it seems like that's most likely a logic error.
        if let Some(mut entity_callbacks) = self.entity_callbacks.get_mut(&entity) {
            entity_callbacks.push(callback_id.clone());
            return;
        }

        // TODO: Make this block an else block on the previous if block once non-lexical scopes are
        // added to Rust.
        {
            let entity_callbacks = vec![callback_id];
            self.entity_callbacks.insert(entity, entity_callbacks);
        }
    }

    fn unregister_all(&mut self, entity: Entity) {
        self.entity_callbacks.remove(&entity);
    }

    /// For a pair of colliding entities A and B, we assume that there is either an entry (A, B) or
    /// (B, A), but not both. We manually invoke the callback for both colliding entities.
    pub fn process_collisions<'a, H>(
        &mut self,
        scene: &Scene,
        collisions: H,
    ) where H: IntoIterator<Item = &'a (Entity, Entity)>{
        let _stopwatch = Stopwatch::new("Process Collision Callbacks");

        {
            let _stopwatch = Stopwatch::new("Sort Collision Data");
            for &(entity, other) in collisions {
                self.entity_collisions.entry(entity).or_insert(Vec::new()).push(other);
                self.entity_collisions.entry(other).or_insert(Vec::new()).push(entity);
            }
        }

        let _stopwatch = Stopwatch::new("Perform collision callbacks");
        for (entity, others) in &mut self.entity_collisions
                                         .iter_mut()
                                         .filter(|&(_, ref others)| others.len() > 0) {
            if let Some(callback_ids) = self.entity_callbacks.get(&entity) {
                for callback_id in callback_ids.iter() {
                    let mut callback = match self.callbacks.get_mut(callback_id) {
                        Some(callback) => callback,
                        None => panic!("No callback with id {:?}", callback_id),
                    };
                    callback.invoke(scene, *entity, &*others);
                }
            }
            others.clear();
        }
    }
}

#[test]
fn sphere_sphere_tests() {
    macro_rules! sphere_test {
        ($lhs:ident, $rhs:ident, $expected:expr) => {
            assert!($rhs.test_sphere(&$lhs) == $expected, "{:#?} vs {:#?} expected {}", $rhs, $lhs, $expected);
            assert!($lhs.test_sphere(&$rhs) == $expected, "{:#?} vs {:#?} expected {}", $lhs, $rhs, $expected);
        }
    }

    let unit = Sphere {
        center: Point::origin(),
        radius: 1.0,
    };


    let far_unit = Sphere {
        center: Point::new(2.0, 2.0, 2.0),
        radius: 1.0,
    };

    let odd_radius = Sphere {
        center: Point::new(1.7, 1.7, 1.7),
        radius: 0.3,
    };

    let unit_edge = Sphere {
        center: Point::new(2.0, 0.0, 0.0),
        radius: 1.0,
    };

    // Identity tests.
    sphere_test!(unit,       unit,       true);
    sphere_test!(far_unit,   far_unit,   true);
    sphere_test!(odd_radius, odd_radius, true);
    sphere_test!(unit_edge,  unit_edge,  true);

    sphere_test!(unit,     far_unit,   false);
    sphere_test!(unit,     odd_radius, false);
    sphere_test!(far_unit, odd_radius, true);
    sphere_test!(unit,     unit_edge,  true);

    // TODO: Centers far from the origin.
    // TODO: Large radius.
    // TODO: Small radius.
}

#[test]
fn obb_obb_tests() {
    use std::f32::consts::PI;

    macro_rules! obb_test {
        ($lhs:ident, $rhs:ident, $expected:expr) => {
            assert!($rhs.test_obb(&$lhs) == $expected, "{:#?} vs {:#?} expected {}", $rhs, $lhs, $expected);
            assert!($lhs.test_obb(&$rhs) == $expected, "{:#?} vs {:#?} expected {}", $lhs, $rhs, $expected);
        }
    }

    let unit = OBB {
        center: Point::origin(),
        orientation: Matrix3::identity(),
        half_widths: Vector3::new(0.5, 0.5, 0.5),
    };

    let rot_z = OBB {
        center: Point::new(1.0, 0.0, 0.0),
        orientation: Matrix3::rotation(0.0, 0.0, 0.25 * PI),
        half_widths: Vector3::new(0.5, 0.5, 0.5),
    };

    obb_test!(unit, unit, true);
    obb_test!(unit, rot_z, true);
}
