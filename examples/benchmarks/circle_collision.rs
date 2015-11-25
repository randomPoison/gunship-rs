extern crate gunship;
extern crate rand;

use std::f32::consts::PI;

use gunship::*;

const TOTAL_CUBES:   usize = 5_000;
const TOTAL_SPHERES: usize = 5_000;

fn main() {
    let mut engine = Engine::new();

    engine.register_system(LoopUpdate { offset: 0.0 });
    engine.register_system(rotation_update);
    engine.register_debug_system(camera_movement);
    setup_scene(engine.scene_mut());

    engine.main_loop();
}

fn setup_scene(scene: &mut Scene) {
    scene.register_manager(LoopMovementManager::new());
    scene.register_manager(RotationManager::new());
    scene.register_manager(CameraControllerManager::new());

    scene.resource_manager().set_resource_path("examples/");
    scene.resource_manager().load_model("meshes/cube.dae").unwrap();
    // scene.resource_manager().load_model("meshes/sphere.dae").unwrap(); // TODO: Fix broken sphere mesh loading.

    let mut transform_manager = scene.get_manager_mut::<TransformManager>();
    let mut camera_manager = scene.get_manager_mut::<CameraManager>();
    let mut light_manager = scene.get_manager_mut::<LightManager>();
    let mut movement_manager = scene.get_manager_mut::<LoopMovementManager>();
    let mut rotation_manager = scene.get_manager_mut::<RotationManager>();
    let mut mesh_manager = scene.get_manager_mut::<MeshManager>();
    let mut collider_manager = scene.get_manager_mut::<ColliderManager>();
    let mut camera_controller = scene.get_manager_mut::<CameraControllerManager>();

    // Create camera.
    {
        let root = scene.create_entity();
        {
            let mut root_transform = transform_manager.assign(root);
            root_transform.set_position(Point::new(0.0, 0.0, 30.0));
            root_transform.look_at(Point::origin(), Vector3::new(0.0, 0.0, -1.0));
        }

        let camera = scene.create_entity();
        transform_manager.assign(camera);
        camera_manager.assign(
            camera,
            Camera::new(
                PI / 3.0,
                1.0,
                0.001,
                100.0));

        transform_manager.set_child(root, camera);

        camera_controller.assign(root, CameraController {
            camera: camera,
            velocity: Vector3::zero(),
        });
    }

    // Create light.
    {
        let light = scene.create_entity();
        transform_manager.assign(light).set_position(Point::new(0.0, 0.0, 5.0));
        light_manager.assign(
            light,
            Light::Point(PointLight {
                position: Point::origin()
            }));
    }

    // Create some amount of cubes.
    for _ in 0..TOTAL_CUBES {
        let entity = scene.create_entity();
        let mut transform =
            transform_manager.assign(entity);
        transform.set_scale(Vector3::new(
            random_range(0.5, 3.5),
            random_range(0.5, 3.5),
            random_range(0.5, 3.5),
        ));
        movement_manager.assign(entity, LoopMovement {
            movement_type: MovementType::CircleZ,
            center: Point::new(
                random_range(-50.0, 50.0),
                random_range(-50.0, 50.0),
                random_range(-50.0, 50.0),
            ),
            radius: random_range(1.0, 5.0),
            period: random_range(10.0, 15.0),
        });
        collider_manager.assign(entity, Collider::Box {
            offset: Vector3::zero(),
            widths: Vector3::one(),
        });
        collider_manager.assign_callback(entity, visualize_collision);
        mesh_manager.assign(entity, "cube.pCube1");
        rotation_manager.assign(entity, RotationMovement::new(
            random_range(0.01, 0.15) * PI,
            random_range(0.01, 0.15) * PI,
            random_range(0.01, 0.15) * PI));
    }

    // Create some amount of spheres.
    for _ in 0..TOTAL_SPHERES {
        let entity = scene.create_entity();
        transform_manager.assign(entity);
        movement_manager.assign(entity, LoopMovement {
            movement_type: MovementType::CircleZ,
            center: Point::new(
                random_range(-50.0, 50.0),
                random_range(-50.0, 50.0),
                random_range(-50.0, 50.0),
            ),
            radius: random_range(1.0, 5.0),
            period: random_range(10.0, 15.0),
        });
        collider_manager.assign(entity, Collider::Sphere {
            offset: Vector3::zero(),
            radius: random_range(0.5, 1.0),
        });
        collider_manager.assign_callback(entity, visualize_collision);
        // mesh_manager.assign(entity, "sphere.pSphere1"); // TODO: Fix loading sphere mesh.
    }

    // { // 5
    //     let entity = scene.create_entity();
    //     let mut transform = transform_manager.assign(entity);
    //     transform.set_position(Point::new(10.0, 9.5, 0.75));
    //     transform.set_scale(Vector3::new(2.0, 1.0, 1.0));
    //     collider_manager.assign(entity, Collider::Box {
    //         offset: Vector3::zero(),
    //         widths: Vector3::one(),
    //     });
    //     collider_manager.assign_callback(entity, visualize_collision);
    //     mesh_manager.assign(entity, "cube.pCube1");
    //
    //     // movement_manager.assign(entity, LoopMovement {
    //     //     movement_type: MovementType::Horizontal,
    //     //     center: Point::new(10.0, 8.0, 0.0),
    //     //     radius: 2.0,
    //     //     period: 10.0,
    //     // });
    // }

    // { // 6
    //     let entity = scene.create_entity();
    //     let mut transform = transform_manager.assign(entity);
    //     transform.set_position(Point::new(11.0, 11.0, 0.0));
    //     transform.set_scale(Vector3::new(1.0, 1.5, 1.0));
    //     collider_manager.assign(entity, Collider::Box {
    //         offset: Vector3::zero(),
    //         widths: Vector3::one(),
    //     });
    //     collider_manager.assign_callback(entity, visualize_collision);
    //     mesh_manager.assign(entity, "cube.pCube1");
    // }

    // { // 6
    //     let entity = scene.create_entity();
    //     let mut transform = transform_manager.assign(entity);
    //     // transform.set_position(Point::new(10.0, 8.5, 0.0));
    //     transform.set_scale(Vector3::new(1.0, 1.0, 1.0));
    //     collider_manager.assign(entity, Collider::Box {
    //         offset: Vector3::zero(),
    //         widths: Vector3::one(),
    //     });
    //     collider_manager.assign_callback(entity, visualize_collision);
    //     mesh_manager.assign(entity, "cube.pCube1");
    //     movement_manager.assign(entity, LoopMovement {
    //         movement_type: MovementType::CircleX,
    //         center: Point::new(10.0, 9.0, 0.75),
    //         radius: 0.75,
    //         period: 6.0,
    //     });
    // }
}

#[derive(Debug, Clone, Copy)]
pub enum MovementType {
    CircleX,
    CircleY,
    CircleZ,
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
struct LoopMovement {
    movement_type: MovementType,
    center:        Point,
    radius:        f32,
    period:        f32,
}

type LoopMovementManager = StructComponentManager<LoopMovement>;

#[derive(Debug, Clone)]
struct LoopUpdate {
    offset: f32,
}

impl System for LoopUpdate {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let movement_manager = scene.get_manager::<LoopMovementManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        self.offset += delta;
        for (movement, entity) in movement_manager.iter_mut() {
            let mut transform = transform_manager.get_mut(entity);

            // Calculate the position of the cube.
            let t = self.offset * PI * 2.0 / movement.period;
            let x_offset = t.cos() * movement.radius;
            let y_offset = t.sin() * movement.radius;

            match movement.movement_type {
                MovementType::CircleX => {
                    transform.set_position(movement.center + Vector3::new(0.0, x_offset, y_offset));
                },
                MovementType::CircleY => {
                    transform.set_position(movement.center + Vector3::new(x_offset, 0.0, y_offset));
                }
                MovementType::CircleZ => {
                    transform.set_position(movement.center + Vector3::new(x_offset, y_offset, 0.0));
                },
                MovementType::Horizontal => {
                    transform.set_position(movement.center + Vector3::new(x_offset, 0.0, 0.0));
                },
                MovementType::Vertical => {
                    transform.set_position(movement.center + Vector3::new(0.0, y_offset, 0.0));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RotationMovement {
    x: f32,
    y: f32,
    z: f32,
}

impl RotationMovement {
    fn new(x: f32, y: f32, z: f32) -> RotationMovement {
        RotationMovement {
            x: x,
            y: y,
            z: z,
        }
    }
}

type RotationManager = StructComponentManager<RotationMovement>;

fn rotation_update(scene: &Scene, delta: f32) {
    let rotation_manager = scene.get_manager::<RotationManager>();
    let transform_manager = scene.get_manager::<TransformManager>();

    for (movement, entity) in rotation_manager.iter() {
        let mut transform = transform_manager.get_mut(entity);
        transform.rotate(Quaternion::from_eulers(movement.x * delta, movement.y * delta, movement.z * delta));
    }
}

fn visualize_collision(scene: &Scene, entity: Entity, _other: &[Entity]) {
    let collider_manager = scene.get_manager::<ColliderManager>();
    collider_manager.bvh_manager().get(entity).unwrap().collider.debug_draw_color(color::RED);
}

#[derive(Debug, Clone, Copy)]
struct CameraController {
    camera: Entity,
    velocity: Vector3,
}

type CameraControllerManager = StructComponentManager<CameraController>;

fn camera_movement(scene: &Scene, delta: f32) {
    const ACCELERATION: f32 = 50.0;
    const MAX_SPEED: f32 = 5.0;

    let player_manager = scene.get_manager::<CameraControllerManager>();
    let transform_manager = scene.get_manager::<TransformManager>();

    for (mut player, root_entity) in player_manager.iter_mut() {

            let (movement_x, movement_y) = scene.input.mouse_delta();

            // Handle movement through root entity.
            // The root entity handles all translation as well as rotation around the Y axis.
            {
                let mut transform = transform_manager.get_mut(root_entity);

                let rotation = transform.rotation();
                let mut velocity = player.velocity;

                transform.set_rotation(Quaternion::from_eulers(0.0, (-movement_x as f32) * PI * 0.001, 0.0) * rotation);

                // Calculate the forward and right vectors.
                // TODO: Directly retrieve local axis from transform without going through rotation matrix.
                let forward_dir = -transform.rotation().as_matrix4().z_part();
                let right_dir = transform.rotation().as_matrix4().x_part();

                // Move camera based on input.
                if scene.input.key_down(ScanCode::W) {
                    velocity = velocity + forward_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::S) {
                    velocity = velocity - forward_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::D) {
                    velocity = velocity + right_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::A) {
                    velocity = velocity - right_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::E) {
                    velocity = velocity + Vector3::up() * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::Q) {
                    velocity = velocity + Vector3::down() * delta * ACCELERATION;
                }

                // Clamp the velocity to the maximum speed.
                if velocity.magnitude() > MAX_SPEED {
                    velocity = velocity.normalized() * MAX_SPEED;
                }

                velocity = velocity * 0.9;
                player.velocity = velocity;
                transform.translate(velocity * delta);
            };

            {
                let mut camera_transform = transform_manager.get_mut(player.camera);
                let rotation = camera_transform.rotation();

                // Apply a rotation to the camera based on mouse movement.
                camera_transform.set_rotation(
                    Quaternion::from_eulers((-movement_y as f32) * PI * 0.001, 0.0, 0.0)
                  * rotation);
            }
    }
}

fn random_range(min: f32, max: f32) -> f32 {
    let range = max - min;
    rand::random::<f32>() * range + min
}
