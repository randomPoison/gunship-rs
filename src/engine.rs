use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::ops::Deref;
use std::collections::HashMap;
use std::intrinsics::type_name;
use std::mem;
use std::ptr;
use std::time::Duration;

use bootstrap;
use bootstrap::input::ScanCode;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time::Timer;
use bs_audio;
use polygon::gl_render::GLRender;
use singleton::Singleton;
use stopwatch::{Collector, Stopwatch};

use scene::*;
use resource::ResourceManager;
use ecs::*;
use component::*;
use debug_draw::DebugDraw;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;
pub const TARGET_FRAME_TIME_MS: f32 = TARGET_FRAME_TIME_SECONDS * 1000.0;

static mut INSTANCE: *mut Engine = 0 as *mut _;

pub struct Engine {
    window: Rc<RefCell<Window>>, // TODO: This doesn't need to be an Rc<RefCell<>> when we're not doing hotloading.
    renderer: Rc<GLRender>,
    resource_manager: Box<ResourceManager>,

    systems: HashMap<SystemId, Box<System>>,
    debug_systems: HashMap<SystemId, Box<System>>,

    // TODO: Replace explicit update ordering with something more automatic (e.g. dependency hierarchy).
    light_update: Box<System>,
    audio_update: Box<System>,
    alarm_update: Box<System>,
    collision_update: Box<System>,

    scene: Scene,

    debug_draw: DebugDraw,

    close: bool,
    debug_pause: bool,
}

impl Engine {
    /// Starts the engine's main loop, blocking until the game shuts down.
    ///
    /// This function starts the engine's internal update loop which handles the details of
    /// processing input from the OS, invoking game code, and rendering each frame. This function
    /// blocks until the engine recieves a message to being the shutdown process, at which point it
    /// will end the update loop and perform any necessary shutdown and cleanup procedures. Once
    /// those have completed this function will return.
    ///
    /// Panics if the engine hasn't been created yet.
    pub fn start() {
        let instance = unsafe {
            debug_assert!(!INSTANCE.is_null(), "Cannot retrieve Engine instance because none exists");
            &mut *INSTANCE
        };

        // Run main loop.
        instance.main_loop();

        // Perform cleanup.
        unsafe { Engine::destroy_instance(); }
    }

    /// Retrieves a reference to current scene.
    ///
    /// Panics if the engine hasn't been created yet.
    pub fn scene<'a>() -> &'a Scene {
        let instance = Engine::instance();
        &instance.scene
    }

    /// Retrieves a reference to the resource manager.
    ///
    /// TODO: The resource manager should probably be a singleton too since it's already setup to
    /// be used through shared references.
    pub fn resource_manager<'a>() -> &'a ResourceManager {
        let instance = Engine::instance();
        &*instance.resource_manager
    }

    fn main_loop(&mut self) {
        let timer = Timer::new();
        let mut collector = Collector::new().unwrap();

        loop {
            let _stopwatch = Stopwatch::new("loop");

            let start_time = timer.now();

            self.update();
            self.draw();

            if self.close {
                break;
            }

            if !cfg!(feature="timing")
            && timer.elapsed_ms(start_time) > TARGET_FRAME_TIME_MS {
                println!("WARNING: Missed frame time. Frame time: {}ms, target frame time: {}ms", timer.elapsed_ms(start_time), TARGET_FRAME_TIME_MS);
            }

            // Wait for target frame time.
            let mut remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            while remaining_time_ms > 1.0 {
                thread::sleep(Duration::from_millis(remaining_time_ms as u64));
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            }

            while remaining_time_ms > 0.0 {
                remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            }

            // TODO: Don't flip buffers until end of frame time?
        };

        collector.flush_to_file("stopwatch.csv");
    }

    fn update(&mut self) {
        let _stopwatch = Stopwatch::new("update");

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

        // TODO: More efficient handling of debug pause (i.e. something that doesn't have any
        // overhead when doing a release build).
        if !self.debug_pause || scene.input.key_pressed(ScanCode::F11) {
            self.debug_draw.clear_buffer();

            self.alarm_update.update(scene, TARGET_FRAME_TIME_SECONDS);

            // Update systems.
            for (_, system) in self.systems.iter_mut() {
                system.update(scene, TARGET_FRAME_TIME_SECONDS);
            }

            // Update component managers.
            scene.update_managers();
        }

        // Update debug systems always forever.
        for (_, system) in self.debug_systems.iter_mut() {
            system.update(scene, TARGET_FRAME_TIME_SECONDS);
        }

        // NOTE: Transform update used to go here.

        if !self.debug_pause || scene.input.key_pressed(ScanCode::F11) {
            self.collision_update.update(scene, TARGET_FRAME_TIME_SECONDS);
            self.light_update.update(scene, TARGET_FRAME_TIME_SECONDS);
            self.audio_update.update(scene, TARGET_FRAME_TIME_SECONDS);
        }

        if scene.input.key_pressed(ScanCode::F9) {
            self.debug_pause = !self.debug_pause;
        }

        if scene.input.key_pressed(ScanCode::F11) {
            self.debug_pause = true;
        }
    }

    #[cfg(not(feature="no-draw"))]
    fn draw(&mut self) {
        let _stopwatch = Stopwatch::new("draw");

        self.renderer.clear();

        let scene = &mut self.scene;
        let camera_manager = scene.get_manager::<CameraManager>();
        let transform_manager = scene.get_manager::<TransformManager>();
        let mesh_manager = scene.get_manager::<MeshManager>();
        let light_manager = scene.get_manager::<LightManager>();

        // Handle rendering for each camera.
        for (camera, entity) in camera_manager.iter() {
            // TODO: Update the camera's bounds in a separate system.
            let camera = {
                let transform = transform_manager.get(entity).unwrap(); // TODO: Don't panic?

                camera.to_polygon_camera(
                    transform.position_derived(),
                    transform.rotation_derived())
            };

            // Draw all of the meshes.
            for (mesh, entity) in mesh_manager.iter() {
                let transform = transform_manager.get(entity).unwrap(); // TODO: Don't panic?

                self.renderer.draw_mesh(
                    &mesh.gl_mesh,
                    &mesh.shader,
                    transform.derived_matrix(),
                    transform.derived_normal_matrix(),
                    &camera,
                    &mut light_manager.iter().map(|(light_ref, _)| *light_ref));
            }

            self.debug_draw.flush_commands(&camera);
        }

        self.renderer.swap_buffers(self.window.borrow().deref());
    }

    #[cfg(feature="no-draw")]
    fn draw(&mut self) {}
}

impl Clone for Engine {
    fn clone(&self) -> Engine {
        let resource_manager = self.resource_manager.clone();

        let engine = Engine {
            window: self.window.clone(),
            renderer: self.renderer.clone(),
            resource_manager: resource_manager.clone(),

            systems: HashMap::new(),
            debug_systems: HashMap::new(),

            light_update: Box::new(LightUpdateSystem),
            audio_update: Box::new(AudioSystem),
            alarm_update: Box::new(alarm_update),
            collision_update: Box::new(CollisionSystem::new()),

            scene: self.scene.clone(),

            debug_draw: DebugDraw::new(self.renderer.clone(), &*resource_manager),

            close: false,
            debug_pause: false,
        };

        engine
    }
}

unsafe impl Singleton for Engine {
    /// Creates the instance of the singleton.
    fn set_instance(engine: Engine) {
        assert!(unsafe { INSTANCE.is_null() }, "Cannot create more than one Engine instance");
        let boxed_engine = Box::new(engine);
        unsafe {
            INSTANCE = Box::into_raw(boxed_engine);
        }
    }

    /// Retrieves an immutable reference to the singleton instance.
    ///
    /// This function is unsafe because there is no way of know
    fn instance() -> &'static Self {
        unsafe {
            debug_assert!(!INSTANCE.is_null(), "Cannot retrieve Engine instance because none exists");
            &*INSTANCE
        }
    }

    /// Destroys the instance of the singleton.
    unsafe fn destroy_instance() {
        let ptr = mem::replace(&mut INSTANCE, ptr::null_mut());
        Box::from_raw(ptr);
    }
}

pub struct EngineBuilder {
    systems: HashMap<SystemId, Box<System>>,
    debug_systems: HashMap<SystemId, Box<System>>,
    managers: ManagerMap,
}

/// A builder for configuring the components and systems registered with the game engine.
///
/// Component managers and systems cannot be changed once the engine has been instantiated so they
/// must be provided all together when the instance is created. `EngineBuilder` provides an
/// interface for gathering all managers and systems to be provided to the engine.
impl EngineBuilder {
    /// Creates a new `EngineBuilder` object.
    pub fn new() -> EngineBuilder {
        let mut builder = EngineBuilder {
            systems: HashMap::new(),
            debug_systems: HashMap::new(),
            managers: ManagerMap::new(),
        };

        // Register internal component managers.
        builder.register_component::<Transform>();
        builder.register_component::<Camera>();
        builder.register_component::<Light>();
        builder.register_component::<Mesh>();
        builder.register_component::<AudioSource>();
        builder.register_component::<AlarmId>();
        builder.register_component::<Collider>();

        builder
    }

    /// Consumes the builder and creates the `Engine` instance.
    ///
    /// No `Engine` object is returned because this method instantiates the engine singleton.
    pub fn build(self) {
        let engine = {
            let instance = bootstrap::init();
            let window = Window::new("Rust Window", instance);
            let renderer = Rc::new(GLRender::new(window.borrow().deref()));
            let resource_manager = Box::new(ResourceManager::new(renderer.clone()));
            let debug_draw = DebugDraw::new(renderer.clone(), &*resource_manager);

            let audio_source = match bs_audio::init() {
                Ok(audio_source) => audio_source,
                Err(error) => {
                    // TODO: Rather than panicking, create a null audio system and keep running.
                    panic!("Error while initialzing audio subsystem: {}", error)
                },
            };

            Engine {
                window: window.clone(),
                renderer: renderer.clone(),
                resource_manager: resource_manager,

                systems: self.systems,
                debug_systems: self.debug_systems,

                light_update: Box::new(LightUpdateSystem),
                audio_update: Box::new(AudioSystem),
                alarm_update: Box::new(alarm_update),
                collision_update: Box::new(CollisionSystem::new()),

                scene: Scene::new(audio_source, self.managers),

                debug_draw: debug_draw,

                close: false,
                debug_pause: false,
            }
        };
        Engine::set_instance(engine);
    }

    /// Registers the manager for the specified component type.
    ///
    /// Defers internally to `register_manager()`.
    pub fn register_component<T: Component>(&mut self) {
        T::Manager::register(self);
    }

    /// Registers the specified manager with the engine.
    ///
    /// Defers internally to `ComponentManager::register()`.

    pub fn register_manager<T: ComponentManager>(&mut self, manager: T) {
        let manager_id = ManagerId::of::<T>();
        assert!(
            !self.managers.contains_key(&manager_id),
            "Manager {} with ID {:?} already registered", unsafe { type_name::<T>() }, &manager_id);

        // Box the manager as a trait object to construct the data and vtable pointers.
        let boxed_manager = Box::new(manager);

        // Add the manager to the type map and the component id to the component map.
        self.managers.insert(manager_id, boxed_manager);
    }

    /// Registers the system with the engine.
    pub fn register_system<T: System>(&mut self, system: T) {
        let system_id = SystemId::of::<T>();

        assert!(
            !self.systems.contains_key(&system_id),
            "System {} with ID {:?} already registered", unsafe { type_name::<T>() }, &system_id);

        self.systems.insert(system_id, Box::new(system));
    }

    /// Registers the debug system with the engine.
    pub fn register_debug_system<T: System>(&mut self, system: T) {
        let system_id = SystemId::of::<T>();

        assert!(
            !self.debug_systems.contains_key(&system_id),
            "System {} with ID {:?} already registered", unsafe { type_name::<T>() }, &system_id);

        self.debug_systems.insert(system_id, Box::new(system));
    }
}

// ==========================
// HOTLOADING MAGIC FUNCTIONS
// ==========================


/*
// FIXME: Hotloading is completely broken for the time being, it's going to need some heavy
// refactoring before it's going to work again.

#[no_mangle]
pub fn engine_init(window: Rc<RefCell<Window>>) -> Box<Engine> {
    let renderer = Rc::new(GLRender::new(window.borrow().deref()));
    let resource_manager = Box::new(ResourceManager::new(renderer.clone()));
    let debug_draw = DebugDraw::new(renderer.clone(), &*resource_manager);

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
        renderer: renderer.clone(),
        resource_manager: resource_manager,

        systems: HashMap::new(),
        debug_systems: HashMap::new(),

        transform_update: Box::new(transform_update),
        light_update: Box::new(LightUpdateSystem),
        audio_update: Box::new(AudioSystem),
        alarm_update: Box::new(alarm_update),
        collision_update: Box::new(CollisionSystem::new()),

        scene: Scene::new(audio_source),

        debug_draw: debug_draw,

        close: false,
        debug_pause: false,
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

// #[cfg(test)] // TODO: Only include this for benchmarks. Double TODO: better support headless benches.
pub fn do_collision_update(engine: &mut Engine) {
    let scene = &mut engine.scene;
    engine.transform_update.update(scene, TARGET_FRAME_TIME_SECONDS);
    engine.collision_update.update(scene, TARGET_FRAME_TIME_SECONDS);
}

*/
