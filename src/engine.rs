use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use bootstrap;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time;

use polygon::gl_render::{self, GLRender};

use scene::Scene;
use resource::ResourceManager;
use ecs::System;
use component::*;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;

pub struct Engine {
    window: Box<Window>,
    renderer: GLRender,
    resource_manager: Rc<RefCell<ResourceManager>>,
    systems: Vec<Box<System>>,
    transform_update: Box<System>,
    light_update: Box<System>,
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
            transform_update: Box::new(TransformUpdateSystem),
            light_update: Box::new(LightUpdateSystem),
            scene: None,
        };

        engine.scene = Some(Scene::new(engine.resource_manager.clone()));

        engine
    }

    pub fn draw(&mut self) {
        self.renderer.clear();

        let scene = self.scene.as_mut().unwrap();
        let mut camera_handle = scene.get_manager::<CameraManager>();
        let mut camera_manager = camera_handle.get();

        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        let mut mesh_handle = scene.get_manager::<MeshManager>();
        let mesh_manager = mesh_handle.get();

        let mut light_handle = scene.get_manager::<LightManager>();
        let mut light_manager = light_handle.get();

        // Handle rendering for each camera.
        for (camera, entity) in camera_manager.iter_mut() {

            // TODO: Update the camera's bounds in a separate system.
            {
                let transform = transform_manager.get(entity);

                camera.position = transform.position;
                camera.rotation = transform.rotation;
            }

            // Draw all of the meshes.
            for (mesh, entity) in mesh_manager.iter() {
                let transform = transform_manager.get_mut(entity);

                self.renderer.draw_mesh(&mesh, transform.matrix(), transform.normal_matrix(), &camera, light_manager.components().as_ref());
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

                self.transform_update.update(scene, frame_time as f32);
                self.light_update.update(scene, frame_time as f32);
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
