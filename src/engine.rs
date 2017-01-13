use camera::CameraData;
use mesh_renderer::MeshRendererData;
use resource::{MaterialId, MeshId};
use scheduler::{self, WorkId};
use transform::{TransformInnerHandle, TransformGraph};
use bootstrap::window::{Message, Window};
use cell_extras::{AtomicInitCell, InitCell};
use input::{self, Input, ScanCode};
use light::LightInner;
use polygon::{GpuMesh, Renderer, RendererBuilder};
use polygon::anchor::Anchor;
use polygon::camera::{Camera as RenderCamera, CameraId};
use polygon::material::MaterialId as PolygonMaterialId;
use polygon::mesh_instance::MeshInstance;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::ptr::{self, Unique};
use std::sync::{Arc, Barrier};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use std::thread;
use stopwatch::{self, stats, PrettyDuration, Stopwatch};

#[derive(Debug)]
pub struct EngineBuilder {
    max_workers: usize,
}

static INSTANCE: AtomicInitCell<Unique<Engine>> = AtomicInitCell::new();
static MAIN_LOOP: AtomicInitCell<WorkId> = AtomicInitCell::new();

/// A builder for configuring the components and systems registered with the game engine.
///
/// Component managers and systems cannot be changed once the engine has been instantiated so they
/// must be provided all together when the instance is created. `EngineBuilder` provides an
/// interface for gathering all managers and systems to be provided to the engine.
impl EngineBuilder {
    /// Creates a new `EngineBuilder` object.
    pub fn new() -> EngineBuilder {
        EngineBuilder {
            max_workers: 1,
        }
    }

    /// Consumes the builder and creates the `Engine` instance.
    ///
    /// `func` is invoked once the engine has been setup, so `func` should kick off all game
    /// functionality.
    ///
    /// No `Engine` object is returned because this method instantiates the engine singleton.
    pub fn build<F>(self, func: F)
        where F: FnOnce()
    {
        let _s = Stopwatch::new("Build engine");

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

                // We're done using the barrier, drop it so that the `Arc` can deallocate once
                // the other thread has receieved the Window.
                mem::drop(barrier_clone);

                message_pump.run();
            });

            // Wait until window thread finishe creating the window.
            barrier.wait();

            window
        };

        // Setup renderer and default shared material.
        let mut renderer = RendererBuilder::new(&window).build();

        let mut material = renderer.default_material();
        material.set_color("surface_color", ::math::Color::rgb(1.0, 0.0, 0.0));
        material.set_color("surface_specular", ::math::Color::rgb(1.0, 1.0, 1.0));
        material.set_f32("surface_shininess", 4.0);
        let default_material_id = renderer.register_shared_material(material);


        let (sender, receiever) = mpsc::channel();

        // Init aysnc subsystem.
        scheduler::init_thread();

        // Spawn our worker threads.
        if self.max_workers > 0 {
            for _ in 0..self.max_workers - 1 {
                let sender = sender.clone();
                thread::spawn(move || {
                    // Initialize thread-local renderer message channel.
                    RENDER_MESSAGE_CHANNEL.with(move |channel| { channel.init(sender); });

                    // Initialize worker thread to support fibers and wait for work to be available.
                    scheduler::run_wait_fiber();
                });
            }
        }

        // Set the current thread's channel.
        RENDER_MESSAGE_CHANNEL.with(move |channel| { channel.init(sender); });

        let mut engine = Box::new(Engine {
            window: window,
            renderer: renderer,
            channel: receiever,

            mesh_map: HashMap::new(),

            scene_graph: TransformGraph::new(),
            lights: Vec::new(),
            camera: None,
            behaviors: Vec::new(),
            input: Input::new(),

            default_material_id: default_material_id,

            debug_pause: false,
        });

        INSTANCE.init(unsafe { Unique::new(&mut *engine) });

        {
            let _s = Stopwatch::new("Scene setup");
            func();
        }

        let main_loop = scheduler::start(move || { main_loop(engine); });

        MAIN_LOOP.init(main_loop.work_id());

        wait_for_quit();

        // Time to shut down the engine.
        let events_string = stopwatch::write_events_to_string();
        let mut out_file = File::create("stopwatch.json").unwrap();
        out_file.write_all(events_string.as_bytes()).unwrap();
    }

    pub fn max_workers(&mut self, workers: usize) -> &mut EngineBuilder {
        assert!(workers > 0, "There must be at least one worker for the engine to run");
        self.max_workers = workers;
        self
    }
}

pub struct Engine {
    window: Window,

    renderer: Box<Renderer>,
    channel: Receiver<EngineMessage>,

    mesh_map: HashMap<MeshId, GpuMesh>,

    scene_graph: TransformGraph,
    lights: Vec<LightInner>,
    camera: Option<(Box<CameraData>, CameraId)>,
    behaviors: Vec<Box<FnMut() + Send>>,
    input: Input,

    default_material_id: PolygonMaterialId,

    debug_pause: bool,
}

impl Drop for Engine {
    fn drop(&mut self) {
        // TODO: Clear instance?
    }
}

// `Engine` does handles synchronization internally, so it's meant be shared between threads.
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

thread_local! {
    // TODO: We don't want this to be completely public, only pub(crate), but `thread_local`
    // doesn't support pub(crate) syntax.
    pub static RENDER_MESSAGE_CHANNEL: InitCell<Sender<EngineMessage>> = InitCell::new();
}

// TODO: This shouln't be public, it's for engine-internal use.
pub fn scene_graph<F, T>(func: F) -> T
    where F: FnOnce(&TransformGraph) -> T
{
    let engine = INSTANCE.borrow();
    unsafe { func(&(***engine).scene_graph) }
}

// TODO: This shouln't be public, it's for engine-internal use.
pub fn input<F, T>(func: F) -> T
    where F: FnOnce(&Input) -> T
{
    let engine = INSTANCE.borrow();
    unsafe { func(&(***engine).input) }
}

// TODO: This shouln't be public, it's for engine-internal use.
pub fn window<F, T>(func: F) -> T
    where F: FnOnce(&Window) -> T
{
    let engine = INSTANCE.borrow();
    unsafe { func(&(***engine).window) }
}

pub enum EngineMessage {
    Anchor(TransformInnerHandle),
    Camera(Box<CameraData>, TransformInnerHandle),
    Light(LightInner),
    Material(MaterialId, ::polygon::material::MaterialSource),
    Mesh(MeshId, ::polygon::geometry::mesh::Mesh),
    MeshInstance(Box<MeshRendererData>, TransformInnerHandle),
    Behavior(Box<FnMut() + Send>),
}

pub fn send_message(message: EngineMessage) {
    RENDER_MESSAGE_CHANNEL.with(move |channel| {
        channel
            .send(message)
            .expect("Unable to send render resource message");
    });
}

pub fn run_each_frame<F>(func: F)
    where
    F: 'static,
    F: FnMut(),
    F: Send,
{
    send_message(EngineMessage::Behavior(Box::new(func)));
}

/// Suspends the calling worker until the engine main loop has finished.
pub fn wait_for_quit() {
    MAIN_LOOP.borrow().await();
}

fn main_loop(mut engine: Box<Engine>) {
    // TODO: This should be a constant, but we can't create constant `Duration` objects right now.
    let target_frame_time = Duration::new(0, 1_000_000_000 / 60);

    let engine = &mut *engine;

    let mut frame_times = Vec::with_capacity(10_000);

    let start_time = Instant::now();
    let mut frame_start = Instant::now();

    'main: loop {
        {
            let _stopwatch = Stopwatch::with_budget("main loop", target_frame_time);

            // Process any pending window messages.
            {
                let _s = Stopwatch::new("Process window messages");
                engine.input.clear();
                for message in &mut engine.window {
                    // TODO: Process input messages.
                    match message {
                        Message::Close => break 'main,
                        Message::Activate => {}, // We don't handle window focus currently.
                        _ => engine.input.push_input(message),
                    }
                }
            }

            if input::key_pressed(ScanCode::F10) {
                engine.debug_pause = !engine.debug_pause;
            }

            let debug_step = input::key_pressed(ScanCode::F11);

            // Kick off all game behaviors and wait for them to complete.
            if engine.behaviors.len() > 0 && (!engine.debug_pause || debug_step) {
                let _stopwatch = Stopwatch::new("game behaviors");
                let mut pending = Vec::with_capacity(engine.behaviors.len());

                // Start all behaviors...
                for behavior in engine.behaviors.iter_mut() {
                    let async = scheduler::start(&mut **behavior);
                    pending.push(async);
                }

                // ... then wait for each of them to finish.
                for async in pending {
                    async.await();
                }
            } else {
                let _s = Stopwatch::new("no game behaviors");
                // There are no per-frame behaviors. We suspend the main loop fiber anyway to give
                // other work some time on the thread. Generally this case only matters when debugging
                // with a single thread.
                scheduler::suspend();
            }

            // Before drawing, process any pending resource messages. These will be resources that were
            // loaded but need to be registered with the renderer before the next draw.
            {
                let _s = Stopwatch::new("Process resource messages");
                while let Ok(message) = engine.channel.try_recv() {
                    match message {
                        EngineMessage::Anchor(transform_inner) => {
                            let _s = Stopwatch::new("Anchor message");
                            let anchor = Anchor::new();
                            let anchor_id = engine.renderer.register_anchor(anchor);

                            transform_inner.set_anchor(anchor_id);
                        },
                        EngineMessage::Camera(camera_data, transform_inner) => {
                            let _s = Stopwatch::new("Camera message");
                            assert!(engine.camera.is_none(), "Can't add camera, one is already registered");

                            let anchor_id = match transform_inner.anchor() {
                                Some(anchor) => anchor,
                                None => unimplemented!(), // TODO: Create the anchor.
                            };

                            let mut camera = RenderCamera::default();
                            camera.set_anchor(anchor_id);
                            let camera_id = engine.renderer.register_camera(camera);

                            engine.camera = Some((camera_data, camera_id));
                        },
                        EngineMessage::Light(light_inner) => {
                            let _s = Stopwatch::new("Light message");
                            {
                                let &(ref id, ref light) = &*light_inner;
                                let light = light.borrow().clone();

                                let light_id = engine.renderer.register_light(light);
                                id.init(light_id);
                            }

                            engine.lights.push(light_inner);
                        }
                        EngineMessage::Material(_material_id, _material_source) => {
                            // let _s = Stopwatch::new("Material message");
                            // let material = engine.renderer.build_material(material_source).expect("TODO: Handle material compilation failure");
                            // let _gpu_material = engine.renderer.register_material(material);
                            //
                            // // TODO: Create an association between `material_id` and `material_source`.
                            unimplemented!();
                        },
                        EngineMessage::Mesh(mesh_id, mesh_data) => {
                            let _s = Stopwatch::new("Mesh message");
                            let gpu_mesh = engine.renderer.register_mesh(&mesh_data);
                            let last = engine.mesh_map.insert(mesh_id, gpu_mesh);
                            assert!(last.is_none(), "Duplicate mesh_id found: {:?}", mesh_id);
                        },
                        EngineMessage::MeshInstance(mesh_renderer_data, transform_inner) => {
                            let _s = Stopwatch::new("Mesh instance message");
                            let anchor_id = match transform_inner.anchor() {
                                Some(anchor) => anchor,
                                None => unimplemented!(), // TODO: Create the anchor.
                            };

                            let gpu_mesh = *engine
                            .mesh_map
                            .get(&mesh_renderer_data.mesh_id())
                            .expect("No gpu mesh found for mesh id");

                            let mut mesh_instance = MeshInstance::with_shared_material(
                                gpu_mesh,
                                engine.default_material_id,
                            );

                            mesh_instance.set_anchor(anchor_id);

                            let _ = engine.renderer.register_mesh_instance(mesh_instance);
                        }
                        EngineMessage::Behavior(func) => {
                            let _s = Stopwatch::new("Behavior message");
                            engine.behaviors.push(func);
                        }
                    }
                }
            }

            // Update renderer's anchors with flattened scene graph.
            {
                let _s = Stopwatch::new("Update renderer anchors");

                for node in engine.scene_graph.roots() {
                    let node = node.borrow();

                    // TODO: Do something like pre-sorting so we only try to update out of
                    // date nodes.
                    if let Some(anchor_id) = node.anchor() {
                        // Send position/rotation/scale to renderer anchor.
                        let anchor = engine.renderer
                        .get_anchor_mut(anchor_id)
                        .expect("Node had anchor id but render did not have specified anchor");
                        anchor.set_position(node.position);
                        anchor.set_orientation(node.orientation);
                        anchor.set_scale(node.scale);
                    }
                }
            }

            // Update the camera.
            if let Some((ref camera_data, ref camera_id)) = engine.camera {
                let _s = Stopwatch::new("Update renderer camera");

                let render_camera = engine.renderer
                    .get_camera_mut(*camera_id)
                    .expect("Camera didn't exist for camera id");

                render_camera.set_fov(camera_data.fov());
                render_camera.set_aspect(camera_data.aspect());
                render_camera.set_near(camera_data.near());
                render_camera.set_far(camera_data.far());
            }

            // Update lights.
            {
                let _s = Stopwatch::new("Update renderer lights");

                for light in &engine.lights {
                    let &(ref id, ref data) = &**light;
                    let light = engine.renderer.get_light_mut(*id.borrow()).expect("Renderer has no such light");
                    *light = data.borrow().clone();
                }
            }

            // Draw.
            engine.renderer.draw();
        }

        frame_times.push(frame_start.elapsed());

        // Determine the next frame's start time, dropping frames if we missed the frame time.
        while frame_start < Instant::now() {
            frame_start += target_frame_time;
        }

        // Now wait until we've returned to the frame cadence before beginning the next frame.
        while Instant::now() < frame_start {
            thread::sleep(Duration::new(0, 0));
        }
    }

    // Print performance statistics.
    // ============================================================================================
    let run_duration = start_time.elapsed();
    let stats = stats::analyze(&*frame_times, target_frame_time);

    println!("Performance statistics:");
    println!("  Duration: {} ({} frames)", PrettyDuration(run_duration), frame_times.len());
    println!("  Min: {}", PrettyDuration(stats.min));
    println!("  Max: {}", PrettyDuration(stats.max));
    println!("  Mean: {}", PrettyDuration(stats.mean));
    println!("  Std: {}", PrettyDuration(stats.std));
    println!("  Long frames: {} ({:.2}%)", stats.long_frames, stats.long_frame_ratio * 100.0);
}
