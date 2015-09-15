use std::collections::{HashMap, HashSet};

use math::*;

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
    grid: HashMap<GridCell, Vec<Entity>>,
    cell_size: f32,
}

impl GridCollisionSystem {
    pub fn new() -> GridCollisionSystem {
        GridCollisionSystem {
            grid: HashMap::new(),
            cell_size: 1.0,
        }
    }

    pub fn update(&mut self, scene: &Scene, _delta: f32) -> HashSet<(Entity, Entity)> {
        println!("GridCollisionSystem::update()");

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

        // Clear out grid contents from previous frame, start each frame with an empty grid an
        // rebuilt it rather than trying to update the grid as objects move.
        for (_, mut cell) in &mut self.grid {
            cell.clear();
        }

        let mut collisions = HashSet::<(Entity, Entity)>::new();

        let bvh_manager = scene.get_manager::<BoundingVolumeManager>();

        for (bvh, entity) in bvh_manager.iter() {
            println!("grid collision test for {:?}", entity);

            let (min, max) = match bvh.root {
                BoundingVolumeNode::Node { ref volume, left_child: _, right_child: _ } => {
                    match volume {
                        &BoundingVolume::AABB { min, max } => {
                            (min, max)
                        },
                        _ => panic!("Bounding volume hierarchy for entity {:?} does not have an AABB at its root, grid collision is only suported with hierarchies that have an AABB at the root"),
                    }
                },
                BoundingVolumeNode::Leaf(_) => panic!("The root of the bounding volume was a leaf node, which is bad and not okay (and probably shouldn't even be possible :sideeye:)"),
            };

            let grid_cell = self.world_to_grid(min);
            let max_cell = self.world_to_grid(max);
            println!("min cell: {:?}, max cell: {:?}", grid_cell, max_cell);

            // Collide against any existing volumes in the
            for test_cell in grid_cell.iter_to(max_cell) {
                println!("testing {:?} in cell {:?}", entity, test_cell);
                if let Some(mut cell) = self.grid.get_mut(&test_cell) {
                    // Check against other volumes.
                    for other_entity in cell.iter() {
                        let other_bvh = bvh_manager.get(*other_entity).unwrap();
                        println!("{:?} and {:?} in the same cell, testing bvhs for collision", bvh.entity, other_entity);
                        if bvh.test(&*other_bvh) {
                            // Woo, we have a collison.
                            println!("legit collision between {:?} and {:?}", bvh.entity, other_bvh.entity);
                            collisions.insert((entity, *other_entity));
                        }
                    }

                    // Add to cell.
                    cell.push(entity);
                    continue;
                }

                // This block should be an else branch on the above if block, but the borrow
                // checker isn't smart enough to tell that the borrow on grid has ended. We do a
                // continue at the end of the previous block so we know if we get here we need to
                // add the cell.
                let cell = vec![entity];
                self.grid.insert(test_cell, cell);
            }

            if let Some(mut cell) = self.grid.get_mut(&grid_cell) {
                cell.push(entity);
                println!("Adding {:?} to cell {:?}", entity, grid_cell);
                continue;
            }

            // This block should be an else branch on the above if block, but the borrow checker
            // isn't smart enough to tell that the borrow on grid has ended. We do a continue at
            // the end of the if block so we can assume if we get here that the cell is not in the
            // grid.
            {
                println!("Creating new cell at {:?} for {:?}", grid_cell, entity);
                self.grid.insert(grid_cell, vec![entity]);
            }
        }

        collisions
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
