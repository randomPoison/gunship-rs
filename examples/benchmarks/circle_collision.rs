extern crate gunship;
extern crate rand;

use std::f32::consts::PI;

use gunship::*;

const TOTAL_CUBES: usize = 10_000;

fn main() {
    let mut engine = Engine::new();

    engine.register_system(circle_movement_update);
    setup_scene(engine.scene_mut());

    engine.main_loop();
}

fn setup_scene(scene: &mut Scene) {
    scene.register_manager(CircleMovementManager::new());

    scene.resource_manager().set_resource_path("examples/");
    scene.resource_manager().load_model("meshes/cube.dae").unwrap();

    let mut transform_manager = scene.get_manager_mut::<TransformManager>();
    let mut camera_manager = scene.get_manager_mut::<CameraManager>();
    let mut light_manager = scene.get_manager_mut::<LightManager>();
    let mut circle_movement_manager = scene.get_manager_mut::<CircleMovementManager>();
    let mut mesh_manager = scene.get_manager_mut::<MeshManager>();
    let mut collider_manager = scene.get_manager_mut::<ColliderManager>();

    // Create camera.
    {
        let camera = scene.create_entity();
        let mut camera_transform = transform_manager.assign(camera);
        camera_transform.set_position(Point::new(0.0, 0.0, 30.0));
        camera_transform.look_at(Point::origin(), Vector3::new(0.0, 0.0, -1.0));
        camera_manager.assign(
            camera,
            Camera::new(
                PI / 3.0,
                1.0,
                0.001,
                100.0));
    }

    // Create light.
    {
        let light = scene.create_entity();
        transform_manager.assign(light);
        light_manager.assign(
            light,
            Light::Point(PointLight {
                position: Point::origin()
            }));
    }

    // Create a marker at the origin.
    {
        let entity = scene.create_entity();
        let mut transform = transform_manager.assign(entity);
        transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        mesh_manager.assign(entity, "cube.pCube1");
    }

    // Create some amount of cubes.
    for _ in 0..TOTAL_CUBES {
        let entity = scene.create_entity();
        transform_manager.assign(entity);
        circle_movement_manager.assign(entity, CircleMovement::new());
        collider_manager.assign(entity, Collider::Sphere {
            offset: Vector3::zero(),
            radius: 0.5,
        });
        collider_manager.register_callback(entity, visualize_collision);
        // collider_manager.register_callback(entity, |_scene: &Scene, _entity, _other_entity| {
        //     println!("collision with the first entity");
        // });
        mesh_manager.assign(entity, "cube.pCube1");
    }
}

#[derive(Debug, Clone)]
struct CircleMovement {
    center: Point,
    radius: f32,
    period: f32,
    offset: f32,
}

impl CircleMovement {
    fn new() -> CircleMovement {
        CircleMovement {
            center: Point::new(
                rand::random::<f32>() * 100.0 - 50.0,
                rand::random::<f32>() * 100.0 - 50.0,
                0.0),
            radius: rand::random::<f32>() * 4.0 + 1.0,
            period: rand::random::<f32>() * 1.0 + 4.0,
            offset: 0.0,
        }
    }
}

type CircleMovementManager = StructComponentManager<CircleMovement>;

fn circle_movement_update(scene: &Scene, delta: f32) {
    let circle_movement_manager = scene.get_manager::<CircleMovementManager>();
    let transform_manager = scene.get_manager::<TransformManager>();

    for (mut circle_movement, entity) in circle_movement_manager.iter_mut() {
        let mut transform = transform_manager.get_mut(entity);

        // Calculate the position of the cube.
        circle_movement.offset += delta;
        let t = circle_movement.offset * PI * 2.0 / circle_movement.period;
        let x_offset = t.cos() * circle_movement.radius;
        let y_offset = t.sin() * circle_movement.radius;

        // Move the cube.
        transform.set_position( circle_movement.center + Vector3::new( x_offset, y_offset, 0.0 ) );
    }
}

fn visualize_collision(scene: &Scene, entity: Entity, _other: Entity) {
    let transform_manager = scene.get_manager::<TransformManager>();
    let transform = transform_manager.get(entity);
    let center = transform.position() + Vector3::new(0.0, 0.0, 0.5);
    debug_draw::box_center_widths(center, Vector3::new(0.4, 0.4, 0.4));
}
