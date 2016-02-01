extern crate gunship;
extern crate rand;

use std::f32::consts::PI;

use gunship::*;

const TOTAL_CUBES: usize = 1000;

fn main() {
    let mut engine = Engine::new();

    engine.register_system(CircleMovementSystem);
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

    // Create some amount of cubes.
    for _ in 0..TOTAL_CUBES {
        let entity = scene.create_entity();
        transform_manager.assign(entity);
        circle_movement_manager.assign(entity, CircleMovement::new());
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
            center: Point::new(rand::random::<f32>() * 30.0 - 15.0, rand::random::<f32>() * 30.0 - 15.0, 0.0),
            radius: rand::random::<f32>() * 4.0 + 1.0,
            period: rand::random::<f32>() * 4.0 + 1.0,
            offset: 0.0,
        }
    }
}

type CircleMovementManager = StructComponentManager<CircleMovement>;

#[derive(Debug, Clone)]
struct CircleMovementSystem;

impl System for CircleMovementSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
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
}
