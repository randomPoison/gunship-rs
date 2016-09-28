use async::*;
use async::resource::{MaterialId, MeshId};
use async::transform::{TransformInnerHandle, TransformGraph};
use bootstrap::window::Window;
use fiber;
use polygon::{GpuMesh, Renderer, RendererBuilder};
use polygon::anchor::Anchor;
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
        scheduler::init();

        // Spawn our worker threads.
        for _ in 0..self.max_workers {
            let sender = sender.clone();
            thread::spawn(move || {
                // Initialize thread-local renderer message channel.
                RENDER_MESSAGE_CHANNEL.with(move |channel| {
                    *channel.borrow_mut() = Some(sender);
                });

                // Initialize worker thread to support fibers and wait for work to be available.
                fiber::init();
                scheduler::wait_for_work();
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
        });

        unsafe { INSTANCE = Some(&*engine); }

        run!({
            loop {
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
                        RenderMessage::Mesh(mesh_id, mesh_data) => {
                            let gpu_mesh = engine.renderer.register_mesh(&mesh_data);
                            println!("sent mesh for {:?} to the gpu: {:?}", mesh_id, gpu_mesh);

                            let last = engine.mesh_map.insert(mesh_id, gpu_mesh);
                            assert!(last.is_none(), "Duplicate mesh_id found: {:?}", mesh_id);
                        },
                        RenderMessage::Material(material_id, material_source) => {
                            let material = engine.renderer.build_material(material_source).expect("TODO: Handle material compilation failure");
                            let gpu_material = engine.renderer.register_material(material);
                            println!("sent material for {:?} to the gpu: {:?}", material_id, gpu_material);

                            // TODO: Create an association between `material_id` and `material_source`.
                        }
                    }
                }

                // TODO: Update renderer's anchors with flattened scene graph.

                // TODO: Draw.

                // TODO: Wait for frame time?
            }
        });

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
    Mesh(MeshId, ::polygon::geometry::mesh::Mesh),
    Material(MaterialId, ::polygon::material::MaterialSource),
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
