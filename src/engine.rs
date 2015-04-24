use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use bootstrap;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time;

use polygon::gl_render::{self, GLRender};

use math::matrix::Matrix4;

use scene::Scene;
use resource::ResourceManager;
use ecs::System;

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

    pub fn scene(&self) -> &Scene {
        self.scene.as_ref().unwrap()
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        self.scene.as_mut().unwrap()
    }
}
