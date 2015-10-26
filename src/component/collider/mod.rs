use std::collections::{HashMap, HashSet};
use std::collections::hash_state::HashState;
use std::cell::{RefCell, Ref, RefMut};

use hash::FnvHashState;
use math::*;
use stopwatch::Stopwatch;

use ecs::*;
use scene::Scene;
use debug_draw;
use super::{EntityMap, EntitySet};
use super::struct_component_manager::{StructComponentManager, ComponentIter};
use self::grid_collision::GridCollisionSystem;
use self::bounding_volume::{BoundingVolumeManager, bvh_update};
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
        widths:  Vector3,
    },

    /// Represents a collision geometry derived from mesh data.
    Mesh,
}

/// Manages the user-facing data in the collision system.
#[derive(Debug, Clone)]
pub struct ColliderManager {
    inner: StructComponentManager<Collider>,
    callback_manager: RefCell<CollisionCallbackManager>,
    bvh_manager: RefCell<BoundingVolumeManager>,
    marked_for_destroy: RefCell<EntitySet>,
}

impl ColliderManager {
    pub fn new() -> ColliderManager {
        ColliderManager {
            inner: StructComponentManager::new(),
            callback_manager: RefCell::new(CollisionCallbackManager::new()),
            bvh_manager: RefCell::new(BoundingVolumeManager::new()),
            marked_for_destroy: RefCell::new(EntitySet::default()),
        }
    }

    pub fn assign(&mut self, entity: Entity, collider: Collider) {
        self.inner.assign(entity, collider);
    }

    pub fn register_callback<T: CollisionCallback + 'static>(&mut self, callback: T) {
        self.callback_manager.borrow_mut().register(callback);
    }

    pub fn assign_callback<T: CollisionCallback + 'static>(&mut self, entity: Entity, callback: T) {
        self.callback_manager.borrow_mut().assign(entity, callback);
    }

    pub fn iter(&self) -> ComponentIter<Collider> {
        self.inner.iter()
    }

    pub fn bvh_manager(&self) -> Ref<BoundingVolumeManager> {
        self.bvh_manager.borrow()
    }

    pub fn bvh_manager_mut(&self) -> RefMut<BoundingVolumeManager> {
        self.bvh_manager.borrow_mut()
    }
}

impl ComponentManager for ColliderManager {
    fn destroy_all(&self, entity: Entity) {
        self.inner.destroy_all(entity);
        self.marked_for_destroy.borrow_mut().insert(entity);
    }

    fn destroy_marked(&mut self) {
        self.inner.destroy_marked();
        let mut marked_for_destroy = self.marked_for_destroy.borrow_mut();
        for entity in marked_for_destroy.drain() {
            self.callback_manager.borrow_mut().unregister_all(entity);
            self.bvh_manager.borrow_mut().destroy_immediate(entity);
        }
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

#[cfg(not(feature="hotloading"))]
type CallbackId = u64;

#[cfg(not(feature="hotloading"))]
fn callback_id<T: CollisionCallback + 'static>() -> CallbackId {
    unsafe { ::std::intrinsics::type_id::<T>() }
}

#[cfg(feature="hotloading")]
type CallbackId = String;

#[cfg(feature="hotloading")]
fn callback_id<T: CollisionCallback + 'static>() -> CallbackId {
    unsafe { ::std::intrinsics::type_name::<T>() }.into()
}

#[derive(Debug)]
pub struct CollisionCallbackManager {
    callbacks: HashMap<CallbackId, Box<CollisionCallback>, FnvHashState>,
    entity_callbacks: EntityMap<Vec<CallbackId>>,
    entity_collisions: EntityMap<Vec<Entity>>,
}

impl CollisionCallbackManager {
    pub fn new() -> CollisionCallbackManager {
        CollisionCallbackManager {
            callbacks: HashMap::default(),
            entity_callbacks: EntityMap::default(),
            entity_collisions: EntityMap::default(),
        }
    }

    fn register<T: CollisionCallback + 'static>(&mut self, callback: T) {
        let callback_id = callback_id::<T>();
        if !self.callbacks.contains_key(&callback_id) {
            self.callbacks.insert(callback_id.clone(), Box::new(callback));
        }
    }

    fn assign<T: CollisionCallback + 'static>(&mut self, entity: Entity, callback: T) {
        let callback_id = callback_id::<T>();
        if !self.callbacks.contains_key(&callback_id) {
            self.callbacks.insert(callback_id.clone(), Box::new(callback));
        }

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
    pub fn process_collisions<H>(
        &mut self,
        scene: &Scene,
        collisions: &HashSet<(Entity, Entity), H>
    ) where H: HashState {
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

impl Clone for CollisionCallbackManager {
    // TODO: Handle re-registering callbacks when cloning.
    fn clone(&self) -> CollisionCallbackManager {
        CollisionCallbackManager {
            callbacks: HashMap::default(),
            entity_callbacks: self.entity_callbacks.clone(),
            entity_collisions: EntityMap::default(),
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
