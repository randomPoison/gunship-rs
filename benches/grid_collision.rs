#![feature(test)]

extern crate gunship;
extern crate hash;
extern crate rand;
extern crate test;

use gunship::*;
use gunship::component::collider::grid_collision::{CollisionGrid, GridCell};
use hash::fnv::FnvHasher;
use self::test::Bencher;
use std::hash::Hash;

macro_rules! random_range {
    ($ty:ty, $min:expr, $max:expr) => {{
        let range = $max - $min;
        rand::random::<$ty>() * range + $min
    }}
}

#[bench]
fn hash_grid_cell_x1000(bencher: &mut Bencher) {
    let mut hasher = FnvHasher::default();

    bencher.iter(|| {
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let grid_cell = test::black_box(GridCell::new(x, y, z));
                    test::black_box(grid_cell.hash(&mut hasher));
                }
            }
        }
    });
}

#[bench]
fn grid_lookup_x1000(bencher: &mut Bencher) {
    // Fill up the grid in advance
    let mut collision_grid = CollisionGrid::default();
    for grid_cell in GridCell::new(-50, -50, -50).iter_to(GridCell::new(50, 50, 50)) {
        collision_grid.insert(grid_cell, Vec::new());
    }

    bencher.iter(|| {
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    test::black_box(collision_grid.get(&GridCell::new(x, y, z)));
                }
            }
        }
    });
}

fn collision_bench(bencher: &mut Bencher) {
    let mut engine = Engine::new();

    {
        let scene = engine.scene_mut();

        scene.resource_manager().set_resource_path("examples/");
        scene.resource_manager().load_model("meshes/cube.dae").unwrap();

        let mut transform_manager = scene.get_manager_mut::<TransformManager>();
        let mut collider_manager = scene.get_manager_mut::<ColliderManager>();

        // Create some amount of cubes.
        for _ in 0..1_000 {
            let entity = scene.create_entity();
            let mut transform = transform_manager.assign(entity);
            transform.set_position(Point::new(
                rand::random::<f32>() * 10.0 - 5.0,
                rand::random::<f32>() * 10.0 - 5.0,
                0.0));
            collider_manager.assign(entity, Collider::Sphere {
                offset: Vector3::zero(),
                radius: 0.5,
            });
            collider_manager.assign_callback(entity, callback);
        }
    }

    bencher.iter(|| {
        ::gunship::engine::do_collision_update(&mut engine);
    });
}

fn callback(_scene: &Scene, _entity: Entity, _other: Entity) {}
