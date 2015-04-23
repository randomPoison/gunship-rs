extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

pub mod entity;
pub mod component;
pub mod system;
pub mod input;
pub mod resource;

use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;

use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::input::ScanCode;
use bootstrap::time;

use math::point::Point;
use math::vector::Vector3;
use math::matrix::Matrix4;

use polygon::gl_render::{self, GLRender};

use entity::EntityManager;
use input::Input;
use component::transform::TransformManager;
use component::camera::CameraManager;
use component::mesh::MeshManager;
use system::System;
use resource::ResourceManager;

pub struct Engine {
    pub window: Box<Window>,
    pub renderer: GLRender,
    pub resource_manager: Rc<RefCell<ResourceManager>>,
    pub entity_manager: EntityManager,
    pub transform_manager: TransformManager,
    pub camera_manager: CameraManager,
    pub mesh_manager: MeshManager,
    pub input: Input,
    systems: Vec<Rc<RefCell<Box<System>>>>
}

impl Engine {
    pub fn new() -> Engine {
        let instance = bootstrap::init();
        let window = Window::new("Rust Window", instance);
        let renderer = gl_render::init(&window);
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(renderer)));

        Engine {
            window: window,
            renderer: renderer,
            resource_manager: resource_manager.clone(),
            entity_manager: EntityManager::new(),
            transform_manager: TransformManager::new(),
            camera_manager: CameraManager::new(),
            mesh_manager: MeshManager::new(resource_manager.clone()),
            input: Input::new(),
            systems: Vec::new()
        }
    }

    pub fn draw(&mut self) {
        self.renderer.clear();

        // Handle rendering for each camera.
        for (camera, entity) in self.camera_manager.iter_mut() {
            // Update the camera's bounds based on it's transform.
            // TODO: Update the camera's bounds before rendering.
            {
                let transform = self.transform_manager.get(entity);
                camera.position = transform.position;
                camera.rotation = Matrix4::rotation(transform.rotation.x, transform.rotation.y, transform.rotation.z);
            }

            // Draw all of the meshes.
            for (mesh, entity) in self.mesh_manager.iter() {
                let mut transform = self.transform_manager.get_mut(entity);
                transform.update(); // TODO: Update all transforms before rendering.
                self.renderer.draw_mesh(&mesh, transform.matrix(), &camera);
            }
        }

        self.renderer.swap_buffers();
    }

    pub fn main_loop(&mut self) {
        let mut close = false;
        let frequency = time::frequency();
        let mut last_time = time::now();

        loop {
            let time_now = time::now();
            let elapsed_time = (time_now - last_time) as f64 / frequency as f64;
            last_time = time_now;

            self.window.handle_messages();
            self.input.clear();
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
                          | MouseWheel(_) => self.input.push_input(message),
                        }
                    },
                    None => break
                }
            }

            // Update systems.
            for system in self.systems.clone().iter_mut() {
                system.borrow_mut().update(self, elapsed_time as f32);
            }

            self.draw();

            if close {
                break;
            }
        };
    }

    pub fn register_system(&mut self, system: Rc<RefCell<Box<System>>>) {
        self.systems.push(system);
    }
}

fn main() {
    // Start Gunship.
    let mut engine = Engine::new();

    // Create camera.
    {
        let camera_entity = engine.entity_manager.create();
        let mut transform = engine.transform_manager.create(camera_entity);
        transform.position = Point::new(5.0, 0.0, 5.0);
        transform.update();
        engine.camera_manager.create(
            camera_entity,
            PI / 3.0,
            1.0,
            0.001,
            100.0);
    }

    // Create gun mesh.
    {
        let mesh_entity = engine.entity_manager.create();
        let mesh_transform = engine.transform_manager.create(mesh_entity);
        mesh_transform.position = Point::new(5.0, 5.0, 5.0);
        mesh_transform.rotation = Vector3::new(0.0, PI, 5.0);
        mesh_transform.scale = Vector3::new(0.5, 1.0, 2.0);
        engine.mesh_manager.create(mesh_entity, "meshes/gun_small.dae");
    }

    engine.register_system(Rc::new(RefCell::new(Box::new(CameraMoveSystem {
        rotation_x: 0.0,
        rotation_y: 0.0
    }))));

    engine.main_loop();
}

struct CameraMoveSystem {
    rotation_x: f32,
    rotation_y: f32
}

impl System for CameraMoveSystem {
    fn update(&mut self, engine: &mut Engine, delta: f32) {
        let entity = engine.camera_manager.entities()[0];

        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (position, rotation) = {
            let transform = engine.transform_manager.get_mut(entity);
            let (movement_x, movement_y) = engine.input.mouse_delta();

            // Add mouse movement to total rotation.
            self.rotation_x += (-movement_y as f32) * PI * 0.001;
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
            let forward_dir = -rotation_matrix.z_part();
            let right_dir = rotation_matrix.x_part();

            // Move camera based on input.
            if engine.input.key_down(ScanCode::W) {
                transform.position = transform.position + forward_dir * delta;
            }

            if engine.input.key_down(ScanCode::S) {
                transform.position = transform.position - forward_dir * delta;
            }

            if engine.input.key_down(ScanCode::D) {
                transform.position = transform.position + right_dir * delta;
            }

            if engine.input.key_down(ScanCode::A) {
                transform.position = transform.position - right_dir * delta
            }

            (transform.position, transform.rotation)
        };

        // Maybe shoot some bullets?
        if engine.input.mouse_button_pressed(0) {
            let bullet_entity = engine.entity_manager.create();
            let bullet_transform = engine.transform_manager.create(bullet_entity);
            bullet_transform.position = position;
            bullet_transform.rotation = rotation;
            bullet_transform.scale = Vector3::new(0.5, 0.5, 0.5);
            engine.mesh_manager.create(bullet_entity, "meshes/gun_small.dae");
        }
    }
}
