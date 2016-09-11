use std::thread;
use std::collections::HashMap;
use std::intrinsics::type_name;
use std::mem;
use std::ptr::{self, Unique};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

use bootstrap::input::ScanCode;
use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::time::Timer;
use bs_audio;
use polygon::{Renderer, RendererBuilder};
use singleton::Singleton;
use stopwatch::{Collector, Stopwatch};

use scene::*;
use resource::ResourceManager;
use ecs::*;
use component::*;
use debug_draw::DebugDraw;

pub const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 60.0;
pub const TARGET_FRAME_TIME_MS: f32 = TARGET_FRAME_TIME_SECONDS * 1000.0;

static mut INSTANCE: *mut Engine = ptr::null_mut();

pub struct Engine {
    renderer: Mutex<Box<Renderer>>,
    window: Window,
    resource_manager: Box<ResourceManager>,

    systems: HashMap<SystemId, Box<System>>,
    debug_systems: HashMap<SystemId, Box<System>>,

    // TODO: Replace explicit update ordering with something more automatic (e.g. dependency hierarchy).
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

    pub fn renderer<F, T>(func: F) -> T
        where F: FnOnce(&mut Renderer) -> T,
    {
        let instance = Engine::instance();
        let mut renderer = instance.renderer.lock().expect("Could not acquire lock on renderer mutex");
        func(&mut **renderer)
    }

    pub fn window() -> &'static Window {
        &Engine::instance().window
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
                println!("shutting down engine");
                break;
            }

            if !cfg!(feature="timing") && timer.elapsed_ms(start_time) > TARGET_FRAME_TIME_MS {
                println!(
                    "WARNING: Missed frame time. Frame time: {}ms, target frame time: {}ms",
                    timer.elapsed_ms(start_time),
                    TARGET_FRAME_TIME_MS);
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

        // TODO: Make this an iterator to simplify this loop.
        while let Some(message) = self.window.next_message() {
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

        self.renderer
        .lock()
        .expect("Unable to acquire lock on renderer mutex for drawing")
        .draw();
    }

    #[cfg(feature="no-draw")]
    fn draw(&mut self) {}
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
    max_workers: usize,
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
            max_workers: 1,
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

            let window = {
                let mut window = unsafe { mem::uninitialized() };
                let mut out = unsafe { Unique::new(&mut window as *mut _) };

                let barrier = Arc::new(Barrier::new(2));
                let barrier_clone = barrier.clone();

                thread::spawn(move || {
                    let mut window = Window::new("gunship game").unwrap();

                    let mut message_pump = window.message_pump();

                    // write data out to `window` without dropping the old (uninitialized) value.
                    unsafe { ptr::write(out.get_mut(), window); }

                    // Sync with
                    barrier_clone.wait();

                    message_pump.run();
                });

                // Wait until window thread finishe creating the window.
                barrier.wait();

                window
            };

            let mut renderer = RendererBuilder::new(&window).build();
            let debug_draw = DebugDraw::new(&mut *renderer);

            let resource_manager = Box::new(ResourceManager::new());

            let audio_source = match bs_audio::init() {
                Ok(audio_source) => audio_source,
                Err(error) => {
                    // TODO: Rather than panicking, create a null audio system and keep running.
                    panic!("Error while initialzing audio subsystem: {}", error)
                },
            };

            Engine {
                window: window,
                renderer: Mutex::new(renderer),
                resource_manager: resource_manager,

                systems: self.systems,
                debug_systems: self.debug_systems,

                audio_update: Box::new(AudioSystem),
                alarm_update: Box::new(alarm_update),
                collision_update: Box::new(CollisionSystem::new()),

                scene: Scene::new(audio_source, self.managers),

                debug_draw: debug_draw,

                close: false,
                debug_pause: false,
            }
        };

        // Init aysnc subsystem.
        ::async::init();
        ::async::start_workers(self.max_workers);

        Engine::set_instance(engine);

        run!(Engine::start());
    }

    pub fn max_workers(&mut self, workers: usize) -> &mut EngineBuilder {
        assert!(workers > 0, "There must be at least one worker for the engine to run");
        self.max_workers = workers;
        self
    }

    /// Registers the manager for the specified component type.
    ///
    /// Defers internally to `register_manager()`.
    pub fn register_component<T: Component>(&mut self) -> &mut EngineBuilder {
        T::Manager::register(self);
        self
    }

    /// Registers the specified manager with the engine.
    ///
    /// Defers internally to `ComponentManager::register()`.

    pub fn register_manager<T: ComponentManager>(&mut self, manager: T) -> &mut EngineBuilder {
        let manager_id = ManagerId::of::<T>();
        assert!(
            !self.managers.contains_key(&manager_id),
            "Manager {} with ID {:?} already registered", unsafe { type_name::<T>() }, &manager_id);

        // Box the manager as a trait object to construct the data and vtable pointers.
        let boxed_manager = Box::new(manager);

        // Add the manager to the type map and the component id to the component map.
        self.managers.insert(manager_id, boxed_manager);

        self
    }

    /// Registers the system with the engine.
    pub fn register_system<T: System>(&mut self, system: T) -> &mut EngineBuilder {
        let system_id = SystemId::of::<T>();

        assert!(
            !self.systems.contains_key(&system_id),
            "System {} with ID {:?} already registered", unsafe { type_name::<T>() }, &system_id);

        self.systems.insert(system_id, Box::new(system));

        self
    }

    /// Registers the debug system with the engine.
    pub fn register_debug_system<T: System>(&mut self, system: T) -> &mut EngineBuilder {
        let system_id = SystemId::of::<T>();

        assert!(
            !self.debug_systems.contains_key(&system_id),
            "System {} with ID {:?} already registered", unsafe { type_name::<T>() }, &system_id);

        self.debug_systems.insert(system_id, Box::new(system));

        self
    }
}
