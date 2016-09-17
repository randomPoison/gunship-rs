use async::scheduler;
use bootstrap::window::Window;
use polygon::{Renderer, RendererBuilder};
use std::mem;
use std::ptr::{self, Unique};
use std::sync::{Arc, Barrier};
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
    /// No `Engine` object is returned because this method instantiates the engine singleton.
    pub fn build<F, T>(self, func: F) -> T
        where F: FnOnce() -> T
    {
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

                    // We're done using the barrier, drop it so that the `Arc` can deallocate once
                    // the other thread has receieved the Window.
                    mem::drop(barrier_clone);

                    message_pump.run();
                });

                // Wait until window thread finishe creating the window.
                barrier.wait();

                window
            };

            let renderer = RendererBuilder::new(&window).build();

            Engine {
                window: window,
                renderer: renderer,
            }
        };

        // Init aysnc subsystem.
        scheduler::init();
        scheduler::start_workers(self.max_workers);

        let boxed_engine = Box::new(engine);

        unsafe { INSTANCE = Some(&*boxed_engine); }

        run!(start());

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
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { INSTANCE = None; }
    }
}

fn start() {
    unimplemented!();
}
