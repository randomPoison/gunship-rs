use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use bootstrap;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time::Timer;
use bootstrap::gl_utils;

use bs_audio;

use polygon::gl_render::{self, GLRender};

use scene::Scene;
use resource::ResourceManager;
use ecs::System;
use component::*;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;
pub const TARGET_FRAME_TIME_MS: f32 = TARGET_FRAME_TIME_SECONDS * 1000.0;

pub struct Engine {
    window: Box<Window>,
    renderer: GLRender,
    resource_manager: Rc<RefCell<ResourceManager>>,
    systems: Vec<Box<System>>,
    transform_update: Box<System>,
    light_update: Box<System>,
    audio_update: Box<System>,
    scene: Option<Scene>,

    close: bool,
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
            audio_update: Box::new(AudioSystem),
            scene: None,

            close: false,
        };

        let audio_source = match bs_audio::init() {
            Ok(audio_source) => {
                println!("Audio subsystem successfully initialized");
                audio_source
            },
            Err(error) => {
                panic!("Error while initialzing audio subsystem: {}", error)
            },
        };

        engine.scene = Some(Scene::new(&engine.resource_manager, audio_source));

        engine
    }

    pub fn update(&mut self) {
        println!("Engine::update() -- changed");

        let scene = self.scene.as_mut().unwrap();

        scene.input.clear();
        loop {
            let message = self.window.next_message(); // TODO: Make this an iterator to simplify this loop.
            match message {
                Some(message) => {
                    match message {
                        Activate => (),
                        Close => self.close = true,
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
            system.update(scene, TARGET_FRAME_TIME_SECONDS);
        }

        self.transform_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        self.light_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        self.audio_update.update(scene, TARGET_FRAME_TIME_SECONDS);
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
        let light_manager = light_handle.get();

        // Handle rendering for each camera.
        for (camera, entity) in camera_manager.iter_mut() {

            // TODO: Update the camera's bounds in a separate system.
            {
                let transform = transform_manager.get(entity);

                camera.position = transform.position_derived();
                camera.rotation = transform.rotation_derived();
            }

            // Draw all of the meshes.
            for (mesh, entity) in mesh_manager.iter() {
                let transform = transform_manager.get_mut(entity);

                self.renderer.draw_mesh(&mesh, transform.derived_matrix(), transform.derived_normal_matrix(), &camera, light_manager.components().as_ref());
            }
        }

        self.renderer.swap_buffers();
    }

    pub fn main_loop(&mut self) {
        let timer = Timer::new();

        loop {
            let start_time = timer.now();

            println!("having window read in messages");
            self.window.handle_messages();

            self.update();
            self.draw();

            if self.close {
                break;
            }

            // Wait for target frame time.
            let mut remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            while remaining_time_ms > 1.0 {
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
                thread::sleep_ms(remaining_time_ms as u32);
            }

            while remaining_time_ms > 0.0 {
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            }

            // TODO: Don't flip buffers until end of frame time?
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

    pub fn close(&self) -> bool {
        self.close
    }

    /// Reloads the engine's systems and component managers. Used for live code editing.
    fn reload(&mut self) {
        // TODO: Do something.
    }
}

#[no_mangle]
pub fn engine_init(window: Box<Window>) -> Box<Engine> {
    let renderer = gl_render::init(&window);
    let resource_manager = Rc::new(RefCell::new(ResourceManager::new(renderer)));

    let mut engine = Engine {
        window: window,
        renderer: renderer,
        resource_manager: resource_manager.clone(),
        systems: Vec::new(),
        transform_update: Box::new(TransformUpdateSystem),
        light_update: Box::new(LightUpdateSystem),
        audio_update: Box::new(AudioSystem),
        scene: None,

        close: false,
    };

    let audio_source = match bs_audio::init() {
        Ok(audio_source) => {
            println!("Audio subsystem successfully initialized");
            audio_source
        },
        Err(error) => {
            panic!("Error while initialzing audio subsystem: {}", error)
        },
    };

    let scene = Scene::new(&engine.resource_manager, audio_source);
    engine.scene = Some(scene);

    Box::new(engine)
}

#[no_mangle]
pub fn engine_update_and_render(engine: &mut Engine) {
    engine.update();
    engine.draw();
}

#[no_mangle]
pub fn engine_reload(mut engine: Box<Engine>) -> Box<Engine> {
    engine.reload();
    // The proc loader needs to be set from within the DLL otherwise we don't
    // correctly bind to OpenGL on Windows.
    gl_utils::set_proc_loader();
    engine
}

#[no_mangle]
pub extern fn engine_close(engine: &Engine) -> bool {
    engine.close()
}
