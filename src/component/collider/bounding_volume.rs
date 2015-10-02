use std::cmp;
use std::cell::RefCell;
use std::slice::Iter;
use std::iter::Zip;

use math::*;
use stopwatch::Stopwatch;

use component::{TransformManager, EntityMap, EntitySet};
use scene::*;
use ecs::*;
use super::{CachedCollider, Collider, ColliderManager};
use debug_draw;

// TODO: Build a custom BVH manager that automatically constructs hierarchy.
/// A default manager for component types that can be represented as a single struct.
#[derive(Debug, Clone)]
pub struct BoundingVolumeManager {
    components: Vec<BoundingVolumeHierarchy>,
    entities: Vec<Entity>,
    indices: EntityMap<usize>,

    marked_for_destroy: RefCell<EntitySet>,
}

impl BoundingVolumeManager {
    pub fn new() -> BoundingVolumeManager {
        BoundingVolumeManager {
            components: Vec::new(),
            entities: Vec::new(),
            indices: EntityMap::default(),

            marked_for_destroy: RefCell::new(EntitySet::default()),
        }
    }

    pub fn assign(&mut self, entity: Entity, component: BoundingVolumeHierarchy) -> &mut BoundingVolumeHierarchy {
        assert!(!self.indices.contains_key(&entity));

        let index = self.components.len();
        self.components.push(component);
        self.entities.push(entity);
        self.indices.insert(entity, index);

        &mut self.components[index]
    }

    pub fn get(&self, entity: Entity) -> Option<&BoundingVolumeHierarchy> {
        if let Some(index) = self.indices.get(&entity) {
            Some(&self.components[*index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut BoundingVolumeHierarchy> {
        if let Some(index) = self.indices.get(&entity) {
            Some(&mut self.components[*index])
        } else {
            None
        }
    }

    pub fn components(&self) -> &Vec<BoundingVolumeHierarchy> {
        &self.components
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn iter(&self) -> Zip<Iter<BoundingVolumeHierarchy>, Iter<Entity>> {
        self.components.iter().zip(self.entities.iter())
    }

    pub fn destroy_immediate(&mut self, entity: Entity) -> BoundingVolumeHierarchy {
        // Retrieve indices of removed entity and the one it's swapped with.
        let index = self.indices.remove(&entity).unwrap();

        // Remove transform and the associate entity.
        let removed_entity = self.entities.swap_remove(index);
        debug_assert!(removed_entity == entity);

        // Update the index mapping for the moved entity, but only if the one we removed
        // wasn't the only one in the row (or the last one in the row).
        if index != self.entities.len() {
            let moved_entity = self.entities[index];
            self.indices.insert(moved_entity, index);
        }

        // Defer removing the transform until the very end to avoid a bunch of memcpys.
        // Transform is a pretty fat struct so if we remove it, cache it to a variable,
        // and then return it at the end we wind up with 2 or 3 memcpys. Doing it all at
        // once at the end (hopefully) means only a single memcpy.
        self.components.swap_remove(index)
    }
}

impl ComponentManager for BoundingVolumeManager {
    fn destroy_all(&self, entity: Entity) {
        if self.indices.contains_key(&entity) {
            self.marked_for_destroy.borrow_mut().insert(entity);
        }
    }

    fn destroy_marked(&mut self) {
        let mut marked_for_destroy = RefCell::new(EntitySet::default());
        ::std::mem::swap(&mut marked_for_destroy, &mut self.marked_for_destroy);
        let mut marked_for_destroy = marked_for_destroy.into_inner();
        for entity in marked_for_destroy.drain() {
            self.destroy_immediate(entity);
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingVolumeHierarchy {
    pub entity: Entity,
    pub root: BoundingVolumeNode,
    pub aabb: AABB,
}

impl BoundingVolumeHierarchy {
    /// Tests if `other` collides with this BVH.
    pub fn test(&self, other: &BoundingVolumeHierarchy) -> bool {
        let our_volume = match &self.root {
            &BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                volume
            },
            &BoundingVolumeNode::Leaf(_) =>
                unimplemented!(),
        };

        let other_volume = match &other.root {
            &BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                volume
            },
            &BoundingVolumeNode::Leaf(_) =>
                unimplemented!(),
        };

        our_volume.test(other_volume)
    }

    pub fn debug_draw(&self) {
        self.root.debug_draw();
    }
}

#[derive(Debug, Clone)]
pub enum BoundingVolumeNode {
    Node {
        volume: BoundingVolume,
        left_child: Option<Box<BoundingVolumeNode>>,
        right_child: Option<Box<BoundingVolumeNode>>,
    },
    Leaf(CachedCollider),
}

impl BoundingVolumeNode {
    pub fn update(&mut self, transform_manager: &TransformManager) {
        match self {
            &mut BoundingVolumeNode::Node { ref mut volume, ref mut left_child, ref mut right_child } => {
                // First update children, then self.
                if let &mut Some(ref mut child) = left_child {
                    child.update(transform_manager);
                }
                if let &mut Some(ref mut child) = right_child {
                    child.update(transform_manager);
                }

                match volume {
                    &mut BoundingVolume::AABB(ref mut aabb) => {
                        aabb.update_to_children(left_child, right_child);
                    },
                    _ => unimplemented!(),
                }
            },
            &mut BoundingVolumeNode::Leaf(ref mut cached_collider) => {
                // Just update cached collider.
                let transform = transform_manager.get(cached_collider.entity);
                cached_collider.position = transform.position_derived();
                cached_collider.orientation = transform.rotation_derived();
                cached_collider.scale = transform.scale_derived();
            },
        }
    }

    pub fn debug_draw(&self) {
        match self {
            &BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                volume.debug_draw();
            },
            &BoundingVolumeNode::Leaf(_) => {
                unimplemented!();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoundingVolume {
    /// Bounding sphere.
    Sphere(BoundingSphere),

    /// Axis-aligned bounding box.
    AABB(AABB),

    /// Oriented boudning box.
    OBB(OBB),
}

impl BoundingVolume {
    pub fn test(&self, other: &BoundingVolume) -> bool {
        let bounds = match self {
            &BoundingVolume::AABB(aabb) => {
                aabb
            },
            _ => unimplemented!(),
        };

        let other_bounds = match other {
            &BoundingVolume::AABB(aabb) => {
                aabb
            },
            _ => unimplemented!(),
        };

        test_ranges((bounds.min.x, bounds.max.x), (other_bounds.min.x, other_bounds.max.x))
     && test_ranges((bounds.min.y, bounds.max.y), (other_bounds.min.y, other_bounds.max.y))
     && test_ranges((bounds.min.z, bounds.max.z), (other_bounds.min.z, other_bounds.max.z))
    }

    pub fn debug_draw(&self) {
        match self {
            &BoundingVolume::Sphere(_) => {
                unimplemented!();
            },
            &BoundingVolume::AABB(aabb) => {
                debug_draw::box_min_max(aabb.min, aabb.max);
            },
            &BoundingVolume::OBB(_) => {
                unimplemented!();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Point,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point,
    pub max: Point,
}

impl AABB {
    /// Updates the AABB so that it completely bounds its children.
    ///
    /// # Details
    ///
    /// This function assumes that left and right children have already been updated.
    pub fn update_to_children(
        &mut self,
        left: &Option<Box<BoundingVolumeNode>>,
        right: &Option<Box<BoundingVolumeNode>>
    ) {
        // If both children are None then something went wrong.
        debug_assert!(left.is_some() || right.is_some());

        let left = left.as_ref().map(aabb_from_node);
        let right = right.as_ref().map(aabb_from_node);

        // We assume that only one will ever need to be defaulted, so we use Point::min for max and
        // Point::max for min to ensure that the other value's bounds are automatically used.
        let left = left.unwrap_or(AABB {
            min: Point::max(),
            max: Point::min(),
        });
        let right = right.unwrap_or(AABB {
            min: Point::max(),
            max: Point::min(),
        });

        self.min = cmp::min(left.min, right.min);
        self.max = cmp::max(left.max, right.max);

        fn aabb_from_node(node: &Box<BoundingVolumeNode>) -> AABB {
            match &**node {
                &BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                    AABB::from_bounding_volume(volume)
                },
                &BoundingVolumeNode::Leaf(ref collider) => {
                    AABB::from_collider(collider)
                },
            }
        }
    }

    /// Given a cached collider generate an AABB that bounds it.
    pub fn from_collider(cached_collider: &CachedCollider) -> AABB {
        match cached_collider.collider {
            Collider::Sphere { offset, radius } => {
                let center = cached_collider.position + offset;
                let half_width = Vector3::new(radius, radius, radius);
                let min = center - half_width;
                let max = center + half_width;

                AABB {
                    min: min,
                    max: max,
                }
            },
            Collider::Box { offset: _, width: _ } => {
                unimplemented!();
            },
            Collider::Mesh => {
                unimplemented!();
            }
        }
    }

    pub fn from_bounding_volume(volume: &BoundingVolume) -> AABB {
        match volume {
            &BoundingVolume::AABB(aabb) => {
                aabb
            },
            &BoundingVolume::Sphere(BoundingSphere { center, radius }) => {
                let min = center - Vector3::new(radius, radius, radius);
                let max = center + Vector3::new(radius, radius, radius);
                AABB {
                    min: min,
                    max: max,
                }
            },
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OBB {
    pub center: Point,
    pub axes: [Vector3; 3],
    pub half_widths: Vector3,
}

pub fn bvh_update(scene: &Scene, _delta: f32) {
    let _stopwatch = Stopwatch::new("BVH Update");

    let collider_manager = scene.get_manager::<ColliderManager>();
    let transform_manager = scene.get_manager::<TransformManager>();
    let mut bvh_manager = scene.get_manager_mut::<BoundingVolumeManager>();

    for (entity, collider) in collider_manager.iter() {
        let transform = transform_manager.get(entity);

        let cached_collider = CachedCollider {
            position: transform.position_derived(),
            orientation: transform.rotation_derived(),
            scale: transform.scale_derived(),
            collider: *collider,
            entity: entity,
        };

        // TODO: We can avoid branching here if we create the BVH when the collider is created,
        // or at least do something to ensure that they already exist by the time we get here.
        if let Some(mut bvh) = bvh_manager.get_mut(entity) {
            bvh.root.update(&*transform_manager);
            bvh.debug_draw();
            continue;
        }

        // This block should be an `else` branch on the previous if block, but the borrow
        // checker isn't smart enough yet to tell that bvh_manager isn't borrowed anymore. We
        // `continue` at the end of the if block so if we get here we know the bvh isn't in
        // the bvh manager yet.
        {
            // Create and insert new bounding volumes.
            let aabb = AABB::from_collider(&cached_collider);
            let root = BoundingVolumeNode::Node {
                volume: BoundingVolume::AABB(aabb),
                left_child: Some(Box::new(BoundingVolumeNode::Leaf(cached_collider))),
                right_child: None,
            };

            bvh_manager.assign(entity, BoundingVolumeHierarchy {
                entity: entity,
                root: root,
                aabb: aabb,
            });
        }
    }
}

fn test_ranges(first: (f32, f32), second: (f32, f32)) -> bool {
    let (min_a, max_a) = first;
    let (min_b, max_b) = second;

    !( min_a > max_b
    || min_b > max_a
    || max_a < min_b
    || max_b < min_a)
}
