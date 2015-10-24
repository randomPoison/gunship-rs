use std::collections::{HashMap};
use std::collections::hash_map::Entry;

use hash::*;
use math::*;
use stopwatch::Stopwatch;

use ecs::Entity;
use super::bounding_volume::*;
// use debug_draw;

/// A collision processor that partitions the space into a regular grid.
///
/// # TODO
///
/// - Do something to configure the size of the grid.
#[derive(Debug)]
#[allow(raw_pointer_derive)]
pub struct GridCollisionSystem {
    pub grid: HashMap<GridCell, Vec<(Entity, *const BoundVolume)>, FnvHashState>,

    /// This should be a HashSet, but HashSet doesn't have a way to get at entries directly.
    pub collisions: HashMap<(Entity, Entity), (), FnvHashState>,
    pub cell_size: f32,

    candidate_collisions: Vec<(*const BoundVolume, *const BoundVolume)>,
}

impl Clone for GridCollisionSystem {
    /// `GridCollisionSystem` doesn't have any real state between frames, it's only used to reuse
    /// the grid's allocated memory between frames. Therefore to clone it we just invoke
    /// `GridCollisionSystem::new()`.
    fn clone(&self) -> Self {
        GridCollisionSystem::new()
    }
}

impl GridCollisionSystem {
    pub fn new() -> GridCollisionSystem {
        GridCollisionSystem {
            grid: HashMap::default(),
            collisions: HashMap::default(),
            cell_size: 1.0,

            candidate_collisions: Vec::new(),
        }
    }

    pub fn update(&mut self, bvh_manager: &BoundingVolumeManager) {
        let _stopwatch = Stopwatch::new("Grid Collision System");

        // // Debug draw the grid.
        // for i in -50..50 {
        //     let offset = i as f32;
        //     debug_draw::line(
        //         Point::new(offset * self.cell_size, -50.0 * self.cell_size, 0.0),
        //         Point::new(offset * self.cell_size,  50.0 * self.cell_size, 0.0));
        //     debug_draw::line(
        //         Point::new(-50.0 * self.cell_size, offset * self.cell_size, 0.0),
        //         Point::new( 50.0 * self.cell_size, offset * self.cell_size, 0.0));
        // }

        self.collisions.clear();

        self.do_broadphase(bvh_manager);
        self.do_narrowphase();

        // Clear out grid contents from previous frame, start each frame with an empty grid and
        // rebuild it rather than trying to update the grid as objects move.
        for (_, mut cell) in &mut self.grid {
            cell.clear();
        }
    }

    fn do_broadphase(&mut self, bvh_manager: &BoundingVolumeManager) {
        let _stopwatch = Stopwatch::new("Broadphase Testing (Grid Based)");
        for bvh in bvh_manager.components() {
            let entity = bvh.entity;

            // Retrieve the AABB at the root of the BVH.
            let aabb = bvh.aabb;

            let min_cell = self.world_to_grid(aabb.min);
            let max_cell = self.world_to_grid(aabb.max);

            // Iterate over all grid cells that the AABB touches. Test the BVH against any entities
            // that have already been placed in that cell, then add the BVH to the cell, creating
            // new cells as necessary.
            for test_cell in min_cell.iter_to(max_cell) {


                if let Some(mut cell) = self.grid.get_mut(&test_cell) {
                    // Check against other volumes.
                    for (_, other_bvh) in cell.iter().cloned() {
                        self.candidate_collisions.push((bvh as *const _, other_bvh));
                    }

                    // Add to existing cell.
                    cell.push((entity, bvh));
                    continue;
                }
                // else
                {
                    let cell = vec![(entity, bvh as *const _)];
                    self.grid.insert(test_cell, cell);
                }
            }
        }
    }

    fn do_narrowphase(&mut self) {
        let _stopwatch = Stopwatch::new("Narrowphase Testing");
        for (bvh, other_bvh) in self.candidate_collisions.drain(0..) {
            let bvh = unsafe { &*bvh };
            let other_bvh = unsafe { &*other_bvh };
            let collision_pair = (bvh.entity, other_bvh.entity);

            // Check if the collision has already been detected before running the
            // collision test since it's potentially very expensive. We get the entry
            // directly, that way we only have to do one hash lookup.
            match self.collisions.entry(collision_pair) {
                Entry::Vacant(vacant_entry) => {
                    // Collision hasn't already been detected, so do the test.
                    if bvh.test(other_bvh) {
                        // Woo, we have a collison.
                        vacant_entry.insert(());
                    }
                },
                _ => {},
            }
        }
    }

    /// Converts a point in world space to its grid cell.
    fn world_to_grid(&self, point: Point) -> GridCell {
        GridCell {
            x: (point.x / self.cell_size).floor() as GridCoord,
            y: (point.y / self.cell_size).floor() as GridCoord,
            z: (point.z / self.cell_size).floor() as GridCoord,
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
    pub x: GridCoord,
    pub y: GridCoord,
    pub z: GridCoord,
}

// TODO: Using i16 for the grid coordinate makes the hash lookups substantially faster, but it means
//       we'll have to take extra care when mapping world coordinates to grid coordinates. Points
//       outside the representable range should be wrapped around. This will technically lead to
//       more grid collisions, but extras will be culled quickly by the AABB test so it shouldn't
//       be more of a performance hit than what we gained from converting to using i16s.
pub type GridCoord = i16;

impl GridCell {
    pub fn new(x: GridCoord, y: GridCoord, z: GridCoord) -> GridCell {
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
