use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::{f32, mem, thread};
use std::sync::{Arc, Mutex, Condvar};
use std::thread::JoinHandle;

use hash::*;
use math::*;
use stopwatch::Stopwatch;

use ecs::Entity;
use super::bounding_volume::*;
use debug_draw;

const NUM_WORKERS: usize = 4;

pub type CollisionGrid = HashMap<GridCell, Vec<*const BoundVolume>, FnvHashState>;

/// A collision processor that partitions the space into a regular grid.
///
/// # TODO
///
/// - Do something to configure the size of the grid.
pub struct GridCollisionSystem {
    _workers: Vec<JoinHandle<()>>,
    thread_data: Arc<WorkTracker>,
    pub collisions: HashSet<(Entity, Entity), FnvHashState>,

    dummy_worker: Worker, // Used during single-threaded testing.
}

impl GridCollisionSystem {
    pub fn new() -> GridCollisionSystem {
        let thread_data = {
            let mut work_units = Vec::new();
            // TODO: Automatically generate work unit bounds.

            if NUM_WORKERS == 1 {
                work_units.push(WorkUnit::new(AABB {
                    min: Point::min(),
                    max: Point::max(),
                }));
            } else if NUM_WORKERS == 2 {
                work_units.push(WorkUnit::new(AABB {
                    min: Point::min(),
                    max: Point::new(0.0, f32::MAX, f32::MAX),
                }));
                work_units.push(WorkUnit::new(AABB {
                    min: Point::new(0.0, f32::MIN, f32::MIN),
                    max: Point::max(),
                }));
            } else if NUM_WORKERS == 4 {
                work_units.push(WorkUnit::new(AABB {
                    min: Point::min(),
                    max: Point::new(0.0, 0.0, f32::MAX),
                }));
                work_units.push(WorkUnit::new(AABB {
                    min: Point::new(f32::MIN, 0.0, f32::MIN),
                    max: Point::new(0.0, f32::MAX, f32::MAX),
                }));
                work_units.push(WorkUnit::new(AABB {
                    min: Point::new(0.0, f32::MIN, f32::MIN),
                    max: Point::new(f32::MAX, 0.0, f32::MAX),
                }));
                work_units.push(WorkUnit::new(AABB {
                    min: Point::new(0.0, 0.0, f32::MIN),
                    max: Point::max(),
                }));
            }

            Arc::new(WorkTracker {
                pending: (Mutex::new(Vec::new()), Condvar::new()),
                complete: (Mutex::new(work_units), Condvar::new()),
            })
        };

        let mut workers = Vec::new();
        for _ in 0..NUM_WORKERS {
            let thread_data = thread_data.clone();
            workers.push(thread::spawn(move || {
                let mut worker = Worker::new(thread_data);
                worker.start();
            }));
        }

        GridCollisionSystem {
            _workers: workers,
            thread_data: thread_data.clone(),
            collisions: HashSet::default(),

            dummy_worker: Worker::new(thread_data.clone()),
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

        if NUM_WORKERS == 0 {
            // Single-threaded collision detection.

            let thread_data = &*self.thread_data;
            let &(ref complete_lock, _) = &thread_data.complete;

            let mut work_units = complete_lock.lock().unwrap();
            let work_unit = &mut work_units[0];

            // Prepare work unit by giving it a copy of the list of volumes.
            work_unit.volumes.clone_from(bvh_manager.components());

            // Actually do the collision detection.
            self.dummy_worker.do_broadphase(work_unit);
            self.dummy_worker.do_narrowphase(work_unit);

            // Merge collision results back into total.
            for (collision, _) in work_unit.collisions.drain() {
                self.collisions.insert(collision);
            }

            // Visualize the collisions.
            for bvh in &work_unit.volumes {
                if bvh.aabb_intersected.get() {
                    debug_draw::box_min_max_color(bvh.aabb.min, bvh.aabb.max, color::RED);
                } else {
                    debug_draw::box_min_max(bvh.aabb.min, bvh.aabb.max);
                }

                if bvh.collider_intersected.get() {
                    bvh.collider.debug_draw_color(color::RED);
                } else {
                    bvh.collider.debug_draw();
                }
            }
        } else {
            let thread_data = &*self.thread_data;
            let &(ref pending_lock, ref pending_condvar) = &thread_data.pending;
            let &(ref complete_lock, ref complete_condvar) = &thread_data.complete;

            // Convert all completed work units into pending work units, notifying a worker thread for each one.
            {
                let mut pending = pending_lock.lock().unwrap();
                let mut complete = complete_lock.lock().unwrap();

                assert!(complete.len() == NUM_WORKERS, "Expected {} complete work units, found {}", NUM_WORKERS, complete.len());
                for mut work_unit in complete.drain(0..) {
                    work_unit.volumes.clone_from(bvh_manager.components());
                    pending.push(work_unit);

                    pending_condvar.notify_one();
                }
            }

            // Wait until all work units have been completed and returned.
            {
                let mut complete = complete_lock.lock().unwrap();
                while complete.len() < NUM_WORKERS {
                    complete = complete_condvar.wait(complete).unwrap();
                }

                for work_unit in complete.iter_mut() {
                    // Visualize the collisions.
                    for bvh in &work_unit.volumes {
                        if !bvh.aabb.test_aabb(&work_unit.bounds) {
                            continue;
                        }

                        if bvh.aabb_intersected.get() {
                            debug_draw::box_min_max_color(bvh.aabb.min, bvh.aabb.max, color::RED);
                        } else {
                            debug_draw::box_min_max(bvh.aabb.min, bvh.aabb.max);
                        }

                        if bvh.collider_intersected.get() {
                            bvh.collider.debug_draw_color(color::RED);
                        } else {
                            bvh.collider.debug_draw();
                        }
                    }

                    // Merge collision results back into total.
                    for (collision, _) in work_unit.collisions.drain() {
                        self.collisions.insert(collision);
                    }
                }
            }
        }
    }
}

impl Clone for GridCollisionSystem {
    /// `GridCollisionSystem` doesn't have any real state between frames, it's only used to reuse
    /// the grid's allocated memory between frames. Therefore to clone it we just invoke
    /// `GridCollisionSystem::new()`.
    fn clone(&self) -> Self {
        GridCollisionSystem::new()
    }
}

#[derive(Debug)]
#[allow(raw_pointer_derive)]
struct WorkUnit {
    volumes: Vec<BoundVolume>,
    collisions: HashMap<(Entity, Entity), (), FnvHashState>, // This should be a HashSet, but HashSet doesn't have a way to get at entries directly.
    bounds: AABB,
}

impl WorkUnit {
    fn new(bounds: AABB) -> WorkUnit {
        WorkUnit {
            volumes: Vec::new(),
            bounds: bounds,
            collisions: HashMap::default(),
        }
    }
}

struct WorkTracker {
    pending: (Mutex<Vec<WorkUnit>>, Condvar),
    complete: (Mutex<Vec<WorkUnit>>, Condvar),
}

struct Worker {
    thread_data: Arc<WorkTracker>,
    grid: HashMap<GridCell, Vec<*const BoundVolume>, FnvHashState>,
    cell_size: f32,

    candidate_collisions: Vec<(*const BoundVolume, *const BoundVolume)>,
}

impl Worker {
    fn new(thread_data: Arc<WorkTracker>) -> Worker {
        Worker {
            thread_data: thread_data,
            grid: HashMap::default(),
            cell_size: 1.0,
            candidate_collisions: Vec::new(),
        }
    }

    fn start(&mut self) {
        loop {
            // Wait until there's pending work, and take the first available one.
            let mut work = {
                let work_tracker = &*self.thread_data;
                let &(ref lock, ref cvar) = &work_tracker.pending;
                let mut pending_work = lock.lock().unwrap();
                while pending_work.len() == 0 {
                    pending_work = cvar.wait(pending_work).unwrap();
                }

                pending_work.pop().unwrap()
            };

            self.do_broadphase(&work);
            self.do_narrowphase(&mut work);

            // Send completed work back to main thread.
            let work_tracker = &*self.thread_data;
            let &(ref lock, ref cvar) = &work_tracker.complete;
            let mut completed_work = lock.lock().unwrap();
            completed_work.push(work);
            cvar.notify_all();
        }
    }

    fn do_broadphase(&mut self, work: &WorkUnit) {
        // let _stopwatch = Stopwatch::new("Broadphase Testing (Grid Based)");
        for bvh in &work.volumes {
            // Retrieve the AABB at the root of the BVH.
            let aabb = bvh.aabb;

            // Only test volumes that are within the bounds of this work unit's testing area.
            if !aabb.test_aabb(&work.bounds) {
                continue;
            }

            let min_cell = self.world_to_grid(aabb.min);
            let max_cell = self.world_to_grid(aabb.max);

            // Iterate over all grid cells that the AABB touches. Test the BVH against any entities
            // that have already been placed in that cell, then add the BVH to the cell, creating
            // new cells as necessary.
            for test_cell in min_cell.iter_to(max_cell) {
                if let Some(mut cell) = self.grid.get_mut(&test_cell) {
                    // Check against other volumes.
                    for other_bvh in cell.iter().cloned() {
                        self.candidate_collisions.push((bvh, other_bvh));
                    }

                    // Add to existing cell.
                    cell.push(bvh);
                    continue;
                }
                // else
                {
                    let cell = vec![bvh as *const _];
                    self.grid.insert(test_cell, cell);
                }
            }
        }

        // Clear out grid contents from previous frame, start each frame with an empty grid and
        // rebuild it rather than trying to update the grid as objects move.
        for (_, mut cell) in &mut self.grid {
            cell.clear();
        }
    }

    fn do_narrowphase(&mut self, work: &mut WorkUnit) {
        // let _stopwatch = Stopwatch::new("Narrowphase Testing");
        for (bvh, other_bvh) in self.candidate_collisions.drain(0..) {
            let bvh = unsafe { &*bvh };
            let other_bvh = unsafe { &*other_bvh };
            let collision_pair = (bvh.entity, other_bvh.entity);

            // Check if the collision has already been detected before running the
            // collision test since it's potentially very expensive. We get the entry
            // directly, that way we only have to do one hash lookup.
            match work.collisions.entry(collision_pair) {
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

        mem::swap(&mut self.next, &mut next);
        Some(next)
    }
}
