extern crate stopwatch;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::ops::Deref;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::intrinsics;
use std::raw::TraitObject;
use std::mem;

use bootstrap;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time::Timer;

use bs_audio;

use polygon::gl_render::GLRender;

use self::stopwatch::{Collector, Stopwatch};

use scene::Scene;
use resource::ResourceManager;
use ecs::System;
use component::*;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;
pub const TARGET_FRAME_TIME_MS: f32 = TARGET_FRAME_TIME_SECONDS * 1000.0;

pub struct Engine {
    window: Rc<RefCell<Window>>, // TODO: This doesn't need to be an Rc<RefCell<>> when we're not doing hotloading.
    renderer: Rc<GLRender>,
    resource_manager: Rc<ResourceManager>,

    systems: Vec<Box<System>>,
    system_indices: HashMap<TypeId, usize>,
    system_names: HashMap<String, TypeId>,

    transform_update: Box<System>,
    light_update: Box<System>,
    audio_update: Box<System>,
    alarm_update: Box<System>,
    collision_update: Box<System>,

    scene: Scene,

    close: bool,
}

impl Engine {
    pub fn new() -> Engine {
        let instance = bootstrap::init();
        let window = Window::new("Rust Window", instance);
        let renderer = Rc::new(GLRender::new(window.borrow().deref()));
        let resource_manager = Rc::new(ResourceManager::new(renderer.clone()));

        let audio_source = match bs_audio::init() {
            Ok(audio_source) => {
                println!("Audio subsystem successfully initialized");
                audio_source
            },
            Err(error) => {
                panic!("Error while initialzing audio subsystem: {}", error)
            },
        };

        Engine {
            window: window.clone(),
            renderer: renderer,
            resource_manager: resource_manager.clone(),

            systems: Vec::new(),
            system_indices: HashMap::new(),
            system_names: HashMap::new(),

            transform_update: Box::new(TransformUpdateSystem),
            light_update: Box::new(LightUpdateSystem),
            audio_update: Box::new(AudioSystem),
            alarm_update: Box::new(AlarmSystem),
            collision_update: Box::new(CollisionSystem::new()),

            scene: Scene::new(&resource_manager, audio_source),

            close: false,
        }
    }

    pub fn update(&mut self) {
        let _stopwatch = Stopwatch::named("update");

        let scene = &mut self.scene;

        scene.input.clear();
        let mut window = self.window.borrow_mut();
        loop {
            let message = window.next_message(); // TODO: Make this an iterator to simplify this loop.
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

        self.alarm_update.update(scene, TARGET_FRAME_TIME_SECONDS);

        // Update systems.
        for system in self.systems.iter_mut() {
            system.update(scene, TARGET_FRAME_TIME_SECONDS);
        }

        self.transform_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        self.collision_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        self.light_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        self.audio_update.update(scene, TARGET_FRAME_TIME_SECONDS);

        // Cleanup any entities that have been marked for destroy.
        scene.destroy_marked();
    }

    pub fn draw(&mut self) {
        let _stopwatch = Stopwatch::named("draw");

        self.renderer.clear();

        let scene = &mut self.scene;
        let camera_manager = scene.get_manager::<CameraManager>();
        let transform_manager = scene.get_manager::<TransformManager>();
        let mesh_manager = scene.get_manager::<MeshManager>();
        let light_manager = scene.get_manager::<LightManager>();

        // Handle rendering for each camera.
        for (mut camera, entity) in camera_manager.iter_mut() {

            // TODO: Update the camera's bounds in a separate system.
            {
                let transform = transform_manager.get(entity);

                camera.position = transform.position_derived();
                camera.rotation = transform.rotation_derived();
            }

            // Draw all of the meshes.
            for (mesh, entity) in mesh_manager.iter() {
                let transform = transform_manager.get(entity);

                self.renderer.draw_mesh(
                    &mesh,
                    transform.derived_matrix(),
                    transform.derived_normal_matrix(),
                    &camera,
                    &mut light_manager.components().iter().map(|ref_cell| *ref_cell.borrow()));
            }
        }

        self.renderer.swap_buffers(self.window.borrow().deref());
    }

    pub fn main_loop(&mut self) {
        let timer = Timer::new();
        let mut collector = Collector::new().unwrap();

        loop {
            let _stopwatch = Stopwatch::named("loop");

            let start_time = timer.now();

            self.update();
            self.draw();

            if self.close {
                break;
            }

            if timer.elapsed_ms(start_time) > TARGET_FRAME_TIME_MS {
                println!("WARNING: Missed frame time. Frame time: {}ms, target frame time: {}ms", timer.elapsed_ms(start_time), TARGET_FRAME_TIME_MS);
            }

            // Wait for target frame time.
            let mut remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            while remaining_time_ms > 1.0 {
                thread::sleep_ms(remaining_time_ms as u32);
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            }

            while remaining_time_ms > 0.0 {
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            }

            // TODO: Don't flip buffers until end of frame time?
        };

        collector.flush_to_file("stopwatch.csv");
    }

    pub fn register_system<T: Any + System>(&mut self, system: T) {
        let system_id = TypeId::of::<T>();
        assert!(!self.system_indices.contains_key(&system_id),
                "System {} with ID {:?} already registered", type_name::<T>(), system_id);

        let index = self.systems.len();
        self.systems.push(Box::new(system));
        self.system_indices.insert(system_id, index);
        self.system_names.insert(type_name::<T>().into(), system_id);
    }

    pub fn get_system<T: Any + System>(&self) -> &T {
        let system_id = TypeId::of::<T>();
        let index =
            *self.system_indices.get(&system_id)
            .expect(&format!("Trying to retrive system {} but no index found", type_name::<T>()));
        let system = &*self.systems[index];

        unsafe {
            // Get the raw representation of the trait object.
            let to: TraitObject = mem::transmute(system);

            // Extract the data pointer.
            mem::transmute(to.data)
        }
    }

    pub fn get_system_by_name<T: Any + System>(&self) -> &T {
        let system_id = self.system_names.get(type_name::<T>()).unwrap();
        let index =
            *self.system_indices.get(system_id)
            .expect(&format!("Trying to retrive system {} but no index found", type_name::<T>()));
        let system = &*self.systems[index];

        unsafe {
            // Get the raw representation of the trait object.
            let to: TraitObject = mem::transmute(system);

            // Extract the data pointer.
            mem::transmute(to.data)
        }
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn close(&self) -> bool {
        self.close
    }
}

impl Clone for Engine {
    fn clone(&self) -> Engine {
        let resource_manager = self.resource_manager.clone();

        let engine = Engine {
            window: self.window.clone(),
            renderer: self.renderer.clone(),
            resource_manager: resource_manager.clone(),

            systems: Vec::new(),
            system_indices: HashMap::new(),
            system_names: HashMap::new(),

            transform_update: Box::new(TransformUpdateSystem),
            light_update: Box::new(LightUpdateSystem),
            audio_update: Box::new(AudioSystem),
            alarm_update: Box::new(AlarmSystem),
            collision_update: Box::new(CollisionSystem::new()),

            scene: self.scene.clone(&resource_manager),

            close: false,
        };

        engine
    }
}

fn type_name<T>() -> &'static str {
    unsafe {
        intrinsics::type_name::<T>()
    }
}

#[no_mangle]
pub fn engine_init(window: Rc<RefCell<Window>>) -> Box<Engine> {
    let renderer = Rc::new(GLRender::new(window.borrow().deref()));
    let resource_manager = Rc::new(ResourceManager::new(renderer.clone()));

    let audio_source = match bs_audio::init() {
        Ok(audio_source) => {
            println!("Audio subsystem successfully initialized");
            audio_source
        },
        Err(error) => {
            panic!("Error while initialzing audio subsystem: {}", error)
        },
    };

    Box::new(Engine {
        window: window,
        renderer: renderer,
        resource_manager: resource_manager.clone(),

        systems: Vec::new(),
        system_indices: HashMap::new(),
        system_names: HashMap::new(),

        transform_update: Box::new(TransformUpdateSystem),
        light_update: Box::new(LightUpdateSystem),
        audio_update: Box::new(AudioSystem),
        alarm_update: Box::new(AlarmSystem),
        collision_update: Box::new(CollisionSystem::new()),

        scene: Scene::new(&resource_manager, audio_source),

        close: false,
    })
}

#[no_mangle]
pub fn engine_reload(engine: &Engine) -> Box<Engine> {
    let new_engine = engine.clone();
    Box::new(new_engine)
}

#[no_mangle]
pub fn engine_update_and_render(engine: &mut Engine) {
    engine.update();
    engine.draw();
}

#[no_mangle]
pub fn engine_close(engine: &Engine) -> bool {
    engine.close()
}

#[no_mangle]
pub fn engine_drop(engine: Box<Engine>) {
    drop(engine);
}
