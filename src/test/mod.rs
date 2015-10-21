extern crate test;
extern crate rand;

use super::*;
use self::test::Bencher;

#[bench]
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
            collider_manager.register_callback(entity, callback);
        }
    }

    bencher.iter(|| {
        engine::do_collision_update(&mut engine);
    });
}

fn callback(_scene: &Scene, _entity: Entity, _other: Entity) {}
