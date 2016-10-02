use async::*;
use async::camera::CameraData;
use async::mesh_renderer::MeshRendererData;
use async::resource::{MaterialId, MeshId};
use async::scheduler::Fiber;
use async::transform::{TransformInnerHandle, TransformGraph};
use bootstrap::window::{Message, Window};
use polygon::{GpuMesh, Renderer, RendererBuilder};
use polygon::anchor::{Anchor, AnchorId};
use polygon::camera::{Camera as RenderCamera, CameraId};
use polygon::mesh_instance::MeshInstance;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::ptr::{self, Unique};
use std::sync::{Arc, Barrier};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug)]
pub struct EngineBuilder {
    max_workers: usize,
}

static mut INSTANCE: Option<*const Engine> = None;
static mut MAIN_LOOP: Option<Fiber> = None;

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
    /// `func` is invoked once the engine has been setup, and teardown will begin once `func`
    /// returns. Therefore, `func` should kick off all game functionality and block until the game
    /// is ready to exit.
    ///
    /// No `Engine` object is returned because this method instantiates the engine singleton.
    pub fn build<F, T>(self, func: F) -> T
        where F: FnOnce() -> T
    {
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

                println!("walking off end of window pump thread, window is going to close");
            });

            // Wait until window thread finishe creating the window.
            barrier.wait();

            window
        };

        let renderer = RendererBuilder::new(&window).build();
        let (sender, receiever) = mpsc::channel();

        // Init aysnc subsystem.
        scheduler::init_thread();

        // Spawn our worker threads.
        for _ in 0..self.max_workers {
            let sender = sender.clone();
            thread::spawn(move || {
                // Initialize thread-local renderer message channel.
                RENDER_MESSAGE_CHANNEL.with(move |channel| {
                    *channel.borrow_mut() = Some(sender);
                });

                // Initialize worker thread to support fibers and wait for work to be available.
                scheduler::run_wait_fiber();
            });
        }

        // Set the current thread's channel.
        RENDER_MESSAGE_CHANNEL.with(move |channel| {
            *channel.borrow_mut() = Some(sender);
        });

        let mut engine = Box::new(Engine {
            window: window,
            renderer: renderer,
            render_message_channel: receiever,

            mesh_map: HashMap::new(),

            scene_graph: TransformGraph::new(),
            camera: None,
        });

        unsafe { INSTANCE = Some(&*engine); }

        let main_loop = run!({
            let engine = &mut *engine;

            'main: loop {
                // Process any pending window messages.
                for message in &mut engine.window {
                    // TODO: Process input messages.
                    if let Message::Close = message {
                        break 'main;
                    }
                }

                // TODO: Update?

                // Before drawing, process any pending render messages. These will be resources that were
                // loaded but need to be registered with the renderer before the next draw.
                while let Ok(message) = engine.render_message_channel.try_recv() {
                    match message {
                        RenderMessage::Anchor(transform_inner) => {
                            let anchor = Anchor::new();
                            let anchor_id = engine.renderer.register_anchor(anchor);

                            transform_inner.set_anchor(anchor_id);
                            println!("created anchor: {:?} for transform: {:?}", anchor_id, transform_inner);
                        },
                        RenderMessage::Camera(camera_data, transform_inner) => {
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
                        RenderMessage::Material(material_id, material_source) => {
                            let material = engine.renderer.build_material(material_source).expect("TODO: Handle material compilation failure");
                            let gpu_material = engine.renderer.register_material(material);
                            println!("sent material for {:?} to the gpu: {:?}", material_id, gpu_material);

                            // TODO: Create an association between `material_id` and `material_source`.
                        },
                        RenderMessage::Mesh(mesh_id, mesh_data) => {
                            let gpu_mesh = engine.renderer.register_mesh(&mesh_data);
                            let last = engine.mesh_map.insert(mesh_id, gpu_mesh);
                            assert!(last.is_none(), "Duplicate mesh_id found: {:?}", mesh_id);
                        },
                        RenderMessage::MeshInstance(mesh_renderer_data, transform_inner) => {
                            let anchor_id = match transform_inner.anchor() {
                                Some(anchor) => anchor,
                                None => unimplemented!(), // TODO: Create the anchor.
                            };

                            let gpu_mesh = *engine
                                .mesh_map
                                .get(&mesh_renderer_data.mesh_id())
                                .expect("No gpu mesh found for mesh id");

                            let mut mesh_instance = MeshInstance::new(
                                gpu_mesh,
                                engine.renderer.default_material(),
                            );
                            mesh_instance.material_mut().set_color("surface_color", ::math::Color::rgb(1.0, 0.0, 0.0)); // HACK HACK HACK
                            mesh_instance.set_anchor(anchor_id);

                            let _ = engine.renderer.register_mesh_instance(mesh_instance);
                        }
                    }
                }

                // Update renderer's anchors with flattened scene graph.
                for row in engine.scene_graph.rows() {
                    // TODO: Don't deal with the `UnsafeCell` directly here if possible.
                    for node in unsafe { &mut *row.get() } {

                        // TODO: Do something like pre-sorting so we only try to update out of
                        // date nodes.
                        node.update_derived_from_parent();
                        if let Some(anchor_id) = node.anchor() {
                            // Send position/rotation/scale to renderer anchor.
                            let anchor = engine.renderer
                                .get_anchor_mut(anchor_id)
                                .expect("Node had anchor id but render did not have specified anchor");
                            anchor.set_position(node.position_derived);
                            anchor.set_orientation(node.orientation_derived);
                            anchor.set_scale(node.scale_derived);
                        }
                    }
                }

                // Update the camera.
                if let Some((ref camera_data, ref camera_id)) = engine.camera {
                    let render_camera = engine.renderer
                        .get_camera_mut(*camera_id)
                        .expect("Camera didn't exist for camera id");

                    render_camera.set_fov(camera_data.fov());
                    render_camera.set_aspect(camera_data.aspect());
                    render_camera.set_near(camera_data.near());
                    render_camera.set_far(camera_data.far());
                }

                // TODO: Draw.
                engine.renderer.draw();

                // TODO: Wait for frame time?
            }
        });

        println!("main loop fiber: {:?}", main_loop);
        unsafe { MAIN_LOOP = Some(main_loop); }

        func()
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
    render_message_channel: Receiver<RenderMessage>,

    mesh_map: HashMap<MeshId, GpuMesh>,

    scene_graph: TransformGraph,
    camera: Option<(Box<CameraData>, CameraId)>,
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { INSTANCE = None; }
    }
}

thread_local! {
    // TODO: We don't want this to be completely public, only pub(crate), but `thread_local`
    // doesn't support pub(crate) syntax.
    pub static RENDER_MESSAGE_CHANNEL: RefCell<Option<Sender<RenderMessage>>> = RefCell::new(None);
}

pub fn scene_graph<F, T>(func: F) -> T
    where F: FnOnce(&TransformGraph) -> T
{
    let engine = unsafe { &*INSTANCE.expect("Engine instance was `None`") };
    func(&engine.scene_graph)
}

#[derive(Debug)]
pub enum RenderMessage {
    Anchor(TransformInnerHandle),
    Camera(Box<CameraData>, TransformInnerHandle),
    Material(MaterialId, ::polygon::material::MaterialSource),
    Mesh(MeshId, ::polygon::geometry::mesh::Mesh),
    MeshInstance(Box<MeshRendererData>, TransformInnerHandle),
}

pub fn send_render_message(message: RenderMessage) {
    RENDER_MESSAGE_CHANNEL.with(move |channel| {
        let borrow = channel.borrow();
        let channel = borrow.as_ref().expect("Render message channel was `None`");
        channel
            .send(message)
            .expect("Unable to send render resource message");
    });
}

/// Suspends the calling worker until the engine main loop has finished.
pub fn wait_for_quit() {
    unsafe { scheduler::wait_for(MAIN_LOOP.as_ref().expect("`MAIN_LOOP` was `None`").clone()); }
}
