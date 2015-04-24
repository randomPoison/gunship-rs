extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

pub mod ecs;
pub mod component;
pub mod input;
pub mod resource;
pub mod scene;

use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::input::ScanCode;
use bootstrap::time;

use math::point::Point;
use math::vector::Vector3;
use math::matrix::Matrix4;

use polygon::gl_render::{self, GLRender};

use ecs::{EntityManager, System, ComponentManager};
use input::Input;
use component::transform::TransformManager;
use component::camera::CameraManager;
use component::mesh::MeshManager;
use component::struct_component_manager::{StructComponentManager, StructComponent};
use resource::ResourceManager;
use scene::Scene;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;

pub struct Engine {
    window: Box<Window>,
    renderer: GLRender,
    resource_manager: Rc<RefCell<ResourceManager>>,
    systems: Vec<Box<System>>,
    scene: Option<Scene>,
}

impl Engine {
    pub fn new() -> Engine {
        let instance = bootstrap::init();
        let window = Window::new("Rust Window", instance);
        let renderer = gl_render::init(&window);
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(renderer)));

        let mut engine = Engine {
            window: window,
            renderer: renderer,
            resource_manager: resource_manager.clone(),
            systems: Vec::new(),
            scene: None,
        };

        engine.scene = Some(Scene::new(engine.resource_manager.clone()));
        engine
    }

    pub fn draw(&mut self) {
        self.renderer.clear();

        let mut scene = self.scene.as_mut().unwrap();

        // Handle rendering for each camera.
        for (camera, entity) in scene.camera_manager.iter_mut() {
            // Update the camera's bounds based on it's transform.
            // TODO: Update the camera's bounds before rendering.
            {
                let transform = scene.transform_manager.get(entity);
                camera.position = transform.position;
                camera.rotation =
                    Matrix4::rotation(transform.rotation.x, transform.rotation.y, transform.rotation.z)
                  * Matrix4::rotation(0.0, PI, 0.0);
            }

            // Draw all of the meshes.
            for (mesh, entity) in scene.mesh_manager.iter() {
                let mut transform = scene.transform_manager.get_mut(entity);
                transform.update(); // TODO: Update all transforms before rendering.
                self.renderer.draw_mesh(&mesh, transform.matrix(), &camera);
            }
        }

        self.renderer.swap_buffers();
    }

    pub fn main_loop(&mut self) {
        let mut close = false;
        let frequency = time::frequency() as f32;
        let mut last_time = time::now();

        loop {
            let start_time = time::now();
            let frame_time = (start_time - last_time) as f32 / frequency;
            last_time = start_time;

            // Block needed to end the borrow of self.scene before the call to draw().
            {
                let scene = self.scene.as_mut().unwrap();

                self.window.handle_messages();
                scene.input.clear();
                loop {
                    let message = self.window.next_message(); // TODO: Make this an iterator to simplify this loop.
                    match message {
                        Some(message) => {
                            match message {
                                Activate => (),
                                Close => close = true,
                                Destroy => (),
                                Paint => (),

                                // Handle inputs.
                                KeyDown(_)
                              | KeyUp(_)
                              | MouseMove(_, _)
                              | MousePos(_, _)
                              | MouseButtonPressed(_)
                              | MouseButtonReleased(_)
                              | MouseWheel(_) => scene.input.push_input(message),
                            }
                        },
                        None => break
                    }
                }

                // Update systems.
                for system in self.systems.iter_mut() {
                    system.update(scene, frame_time as f32);
                }
            }

            self.draw();

            if close {
                break;
            }

            let end_time = time::now();
            let mut elapsed_time = (end_time - start_time) as f32 / frequency;
            if elapsed_time > 0.1 {
                elapsed_time = 0.1;
            }

            let difference = TARGET_FRAME_TIME_SECONDS - elapsed_time;
            if difference > 0.0 {
                let sleep_ms = (difference * 1000.0).round() as u32;
                thread::sleep_ms(sleep_ms);
            } else {
                println!("Failed to meet required frame time: {} ms", elapsed_time);
            }
        };
    }

    pub fn register_system(&mut self, system: Box<System>) {
        self.systems.push(system);
    }
}

fn main() {
    // Start Gunship.
    let mut engine = Engine::new();

    // Block needed to end borrow of engine.scene before call to register_system().
    {
        let scene = engine.scene.as_mut().unwrap();

        // Create camera.
        {
            let camera_entity = scene.entity_manager.create();
            let mut transform = scene.transform_manager.create(camera_entity);
            transform.position = Point::new(5.0, 0.0, 5.0);
            transform.update();
            scene.camera_manager.create(
                camera_entity,
                PI / 3.0,
                1.0,
                0.001,
                100.0);
        }

        // Create gun mesh.
        {
            let mesh_entity = scene.entity_manager.create();
            let mesh_transform = scene.transform_manager.create(mesh_entity);
            mesh_transform.position = Point::new(5.0, 5.0, 5.0);
            mesh_transform.rotation = Vector3::new(0.0, PI, 5.0);
            scene.mesh_manager.create(mesh_entity, "meshes/gun_small.dae");
        }

        scene.register_manager::<BulletManager>(Box::new(StructComponentManager::new()));
    }

    engine.register_system(Box::new(CameraMoveSystem {
        rotation_x: 0.0,
        rotation_y: 0.0,
    }));
    engine.register_system(Box::new(BulletSystem));

    engine.main_loop();
}

struct CameraMoveSystem {
    rotation_x: f32,
    rotation_y: f32
}

impl System for CameraMoveSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let entity = scene.camera_manager.entities()[0];

        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (position, rotation) = {
            let transform = scene.transform_manager.get_mut(entity);
            let (movement_x, movement_y) = scene.input.mouse_delta();

            // Add mouse movement to total rotation.
            self.rotation_x += (movement_y as f32) * PI * 0.001;
            self.rotation_y += (-movement_x as f32) * PI * 0.001;

            // Apply a rotation to the camera based on mouse movmeent.
            transform.rotation =
                Vector3::new(self.rotation_x,
                             self.rotation_y,
                             0.0);
            let rotation_matrix =
                Matrix4::rotation(self.rotation_x,
                                  self.rotation_y,
                                  0.0);

            // Calculate the forward and right vectors.
            let forward_dir = rotation_matrix.z_part();
            let right_dir = -rotation_matrix.x_part();

            // Move camera based on input.
            if scene.input.key_down(ScanCode::W) {
                transform.position = transform.position + forward_dir * delta;
            }

            if scene.input.key_down(ScanCode::S) {
                transform.position = transform.position - forward_dir * delta;
            }

            if scene.input.key_down(ScanCode::D) {
                transform.position = transform.position + right_dir * delta;
            }

            if scene.input.key_down(ScanCode::A) {
                transform.position = transform.position - right_dir * delta
            }

            (transform.position, transform.rotation)
        };

        // Maybe shoot some bullets?
        if scene.input.mouse_button_pressed(0) {
            let bullet_entity = scene.entity_manager.create();

            // Block is needed to end borrow of scene.transform_manager
            // before scene.get_manager_mut() can be called.
            {
                let bullet_transform = scene.transform_manager.create(bullet_entity);
                bullet_transform.position = position;
                bullet_transform.rotation = rotation;
                bullet_transform.scale = Vector3::new(0.5, 0.5, 0.5);
                scene.mesh_manager.create(bullet_entity, "meshes/bullet_small.dae");
            }

            let mut bullet_handle = scene.get_manager_mut::<BulletManager>();
            let mut bullet_manager = bullet_handle.get();
            bullet_manager.create(bullet_entity);
        }
    }
}

struct Bullet {
    direction: Vector3,
    speed: f32,
}

pub type BulletManager = StructComponentManager<Bullet>;

impl StructComponent for Bullet {
    fn new() -> Bullet {
        Bullet {
            direction: Vector3::one(),
            speed: 1.0,
        }
    }
}

struct BulletSystem;

impl System for BulletSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let mut bullet_handle = scene.get_manager_mut::<BulletManager>();
        let mut bullet_manager = bullet_handle.get();
        for (bullet, entity) in bullet_manager.iter() {
            let mut transform = scene.transform_manager.get_mut(entity);
            let forward = transform.rotation_matrix().z_part();
            transform.position = transform.position + forward * bullet.speed * delta;
        }
    }
}
