use std::collections::{HashMap, HashSet};

use math::*;
use fnv::FnvHashState;

use scene::Scene;
use ecs::Entity;
use super::bounding_volume::*;
use debug_draw;

/// A collision processor that partitions the space into a regular grid.
///
/// # TODO
///
/// - Do something to configure the size of the grid.
#[derive(Debug, Clone)]
pub struct GridCollisionSystem {
    pub grid: HashMap<GridCell, Vec<*const BoundingVolumeHierarchy>, FnvHashState>,
    pub collisions: HashSet<(Entity, Entity), FnvHashState>,
    pub cell_size: f32,
}

impl GridCollisionSystem {
    pub fn new() -> GridCollisionSystem {
        GridCollisionSystem {
            grid: HashMap::default(),
            collisions: HashSet::default(),
            cell_size: 1.0,
        }
    }

    pub fn update(&mut self, scene: &Scene, _delta: f32) {
        // Debug draw the grid.
        for i in -50..50 {
            let offset = i as f32;
            debug_draw::line(
                Point::new(offset * self.cell_size, -50.0 * self.cell_size, 0.0),
                Point::new(offset * self.cell_size,  50.0 * self.cell_size, 0.0));
            debug_draw::line(
                Point::new(-50.0 * self.cell_size, offset * self.cell_size, 0.0),
                Point::new( 50.0 * self.cell_size, offset * self.cell_size, 0.0));
        }

        let bvh_manager = scene.get_manager::<BoundingVolumeManager>();
        self.collisions.clear();

        for (bvh, entity) in bvh_manager.iter() {
            let entity = *entity;

            // Retrieve the AABB at the root of the BVH.
            let aabb = match bvh.root {
                BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                    match volume {
                        &BoundingVolume::AABB(aabb) => {
                            aabb
                        },
                        _ => panic!("Bounding volume hierarchy for entity {:?} does not have an AABB at its root, grid collision is only suported with hierarchies that have an AABB at the root", entity),
                    }
                },
                BoundingVolumeNode::Leaf(_) => panic!("The root of the bounding volume was a leaf node, which is bad and not okay (and probably shouldn't even be possible :sideeye:)"),
            };

            let grid_cell = self.world_to_grid(aabb.min);
            let max_cell = self.world_to_grid(aabb.max);

            // Collide against any existing volumes in the
            for test_cell in grid_cell.iter_to(max_cell) {
                if let Some(mut cell) = self.grid.get_mut(&test_cell) {
                    // Check against other volumes.
                    for other_bvh in cell.iter().cloned().map(|bvh_ptr| -> &BoundingVolumeHierarchy { unsafe { &*bvh_ptr } }) {
                        if self.collisions.contains(&(entity, other_bvh.entity)) {
                            // Collision already detected, no need to check again.
                            continue;
                        }

                        if bvh.test(other_bvh) {
                            // Woo, we have a collison.
                            self.collisions.insert((entity, other_bvh.entity));
                        }
                    }

                    // Add to cell.
                    cell.push(bvh);
                    continue;
                }

                // This block should be an else branch on the above if block, but the borrow
                // checker isn't smart enough to tell that the borrow on grid has ended. We do a
                // continue at the end of the previous block so we know if we get here we need to
                // add the cell.
                let cell = vec![bvh as *const _];
                self.grid.insert(test_cell, cell);
            }

            if let Some(mut cell) = self.grid.get_mut(&grid_cell) {
                cell.push(bvh);
                continue;
            }

            // This block should be an else branch on the above if block, but the borrow checker
            // isn't smart enough to tell that the borrow on grid has ended. We do a continue at
            // the end of the if block so we can assume if we get here that the cell is not in the
            // grid.
            {
                self.grid.insert(grid_cell, vec![bvh as *const _]);
            }
        }

        // Clear out grid contents from previous frame, start each frame with an empty grid an
        // rebuilt it rather than trying to update the grid as objects move.
        for (_, mut cell) in &mut self.grid {
            cell.clear();
        }
    }

    /// Converts a point in world space to its grid cell.
    fn world_to_grid(&self, point: Point) -> GridCell {
        GridCell {
            x: (point.x / self.cell_size).floor() as isize,
            y: (point.y / self.cell_size).floor() as isize,
            z: (point.z / self.cell_size).floor() as isize,
        }
    }
}

/// A wrapper type around a triple of coordinates that uniquely identify a grid cell.
///
/// # Details
///
/// Grid cells are axis-aligned cubes of a regular sice. The coordinates of a grid cell are its min
/// value. This was chosen because of how it simplifies the calculation to find the cell for a
/// given point (`(point / cell_size).floor()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl GridCell {
    pub fn new(x: isize, y: isize, z: isize) -> GridCell {
        GridCell {
            x: x,
            y: y,
            z: z,
        }
    }

    pub fn iter_to(&self, dest: GridCell) -> GridIter {
        // assert!(self < dest, "start point for grid iter must be less that end point, or use iter_from()");

        GridIter {
            from: *self,
            to:   dest,
            next: *self,
        }
    }
}

pub struct GridIter {
    from: GridCell,
    to:   GridCell,
    next: GridCell,
}

impl Iterator for GridIter {
    type Item = GridCell;

    fn next(&mut self) -> Option<GridCell> {
        let from = self.from;
        let to = self.to;
        let mut next = self.next;

        if next.z >= to.z {
            next.z = from.z;
            if next.y >= to.y {
                next.y = from.y;
                if next.x >= to.x {
                    return None;
                } else {
                    next.x += 1;
                }
            } else {
                next.y += 1;
            }
        } else {
            next.z += 1;
        }

        ::std::mem::swap(&mut self.next, &mut next);
        Some(next)
    }
}
