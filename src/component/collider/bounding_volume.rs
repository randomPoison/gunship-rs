use math::*;

use component::{TransformManager, StructComponentManager};
use scene::*;
use ecs::*;
use super::{CachedCollider, Collider, ColliderManager};
use debug_draw;

// TODO: Build a custom BVH manager that automatically constructs hierarchy.
pub type BoundingVolumeManager = StructComponentManager<BoundingVolumeHierarchy>;

#[derive(Debug, Clone)]
pub struct BoundingVolumeHierarchy {
    pub entity: Entity,
    pub root: BoundingVolumeNode,
}

impl BoundingVolumeHierarchy {
    /// Traverses the hierarchy and updates the bounding volumes to match any changes
    /// in the root colliders.
    pub fn update(&mut self, _transform_manager: &TransformManager) {
        // self.root.update(transform_manager);
    }

    /// Tests if `other` collides with this BVH.
    pub fn test(&self, _other: &BoundingVolumeHierarchy) -> bool {
        true // TODO: Actually test the collision.
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
            &mut BoundingVolumeNode::Node { volume: _, left_child: _, right_child: _ } => {
                // uhhh
                unimplemented!();
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

#[derive(Debug, Clone)]
pub enum BoundingVolume {
    /// Bounding sphere.
    Sphere {
        center: Point,
        radius: f32,
    },

    /// Axis-aligned bounding box.
    AABB {
        min: Point,
        max: Point,
    },

    /// Oriented boudning box.
    OBB {
        center: Point,
        axes: [Vector3; 3],
        half_widths: Vector3,
    },
}

impl BoundingVolume {
    /// Given a cached collider generate an AABB that bounds it.
    pub fn generate_aabb(cached_collider: &CachedCollider) -> BoundingVolume {
        match cached_collider.collider {
            Collider::Sphere { offset, radius } => {
                let center = cached_collider.position + offset;
                let half_width = Vector3::new(radius, radius, radius);
                let min = center - half_width;
                let max = center + half_width;

                BoundingVolume::AABB {
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

    pub fn debug_draw(&self) {
        match self {
            &BoundingVolume::Sphere { center: _, radius: _ } => {
                unimplemented!();
            },
            &BoundingVolume::AABB { min, max } => {
                debug_draw::box_min_max(min, max);
            },
            &BoundingVolume::OBB { center: _, axes: _, half_widths: _ } => {
                unimplemented!();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingVolumeUpdateSystem;

impl System for BoundingVolumeUpdateSystem {
    fn update(&mut self, scene: &Scene, _delta: f32) {
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

            if let Some(mut bvh) = bvh_manager.get_mut(entity) {
                bvh.update(&*transform_manager);
                bvh.debug_draw();
                continue;
            }

            // This block should be an `else` branch on the previous if block, but the borrow
            // checker isn't smart enough yet to tell that bvh_manager isn't borrowed anymore. We
            // `continue` at the end of the if block so if we get here we know the bvh isn't in
            // the bvh manager yet.
            {
                // Create and insert new bounding volumes.
                let root = BoundingVolumeNode::Node {
                    volume: BoundingVolume::generate_aabb(&cached_collider),
                    left_child: Some(Box::new(BoundingVolumeNode::Leaf(cached_collider))),
                    right_child: None,
                };

                bvh_manager.assign(entity, BoundingVolumeHierarchy {
                    entity: entity,
                    root: root,
                });
            }
        }
    }
}
