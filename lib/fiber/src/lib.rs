//! A libray for creating and managing fibers in a cross-platform manner.
//!
//! Fibers are threads that must be manually scheduled by the client application, as opposed to
//! threads which are automatically managed and scheduled by the OS. Each fiber has its own stack
//! space and can be yield its time to another thread at any point during execution. This allows
//! for different forms of concurrency to be implemented in a way that's optimal for the client
//! application.
//!
//! This library is meant to be the base for a fiber-pool system, in which a fixed number of worker
//! fibers are created and used to asynchronously complete units of work.

#![feature(raw)]

use platform::PlatformId;
use std::cell::Cell;

#[cfg(target_os="windows")]
#[path="platform\\windows.rs"]
pub mod platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiberId(PlatformId);

/// Represents a fiber with its own stack and thread state.
///
/// The fiber's lifetime is associated with any data it borrows when created.
#[derive(Debug)]
pub struct Fiber(PlatformId);

/// A global cache mapping threads to their currently running fiber.
///
/// This is used by `Fiber::current()` on some platforms to keep track of which fiber is active
/// on which thread.
thread_local! {
    static PREV: Cell<Option<PlatformId>> = Cell::new(None);
    static CURRENT: Cell<Option<PlatformId>> = Cell::new(None);
}

/// Initializes the current thread, making it safe to begin using threads.
///
/// On some platforms initialization is required before using threads (e.g. on Windows versions
/// older than 7 the main thread must be converted to a fiber on startup). This function performs
/// any necessary initialization and returns the active fiber. This function must be called for all
/// spawned threads to ensure that it is safe to use fibers from those threads.
pub fn init() -> FiberId {
    let platform_fiber = platform::init();

    // Initialize our thread-local cache of the current fiber.
    CURRENT.with(|current| current.set(Some(platform_fiber)));

    FiberId(platform_fiber)
}

impl Fiber {
    /// Creates a new fiber with the specified stack size and has it begin executing the specified
    /// function.
    ///
    /// # Panics
    ///
    /// Panics if `init()` has not yet been called on the current thread.
    pub fn new<F>(stack_size: usize, func: F) -> Fiber
        where
        F: Fn(Fiber),
        F: 'static + Send,
    {
        Fiber(platform::create_fiber(stack_size, func))
    }

    /// Makes the fiber active, consuming in the process.
    ///
    /// This suspends the current fiber so that the resumed fiber can run in its place. At a later
    /// point another fiber may resume the current one, at which point `resume()` with return,
    /// yielding the fiber that was suspended.
    pub unsafe fn resume(self) -> Fiber {
        // Initialize the current thread for fiber usage if we haven't done so already.
        if let None = CURRENT.with(|current| current.get()) {
            init();
        }

        {
            let prev_handle = CURRENT.with(|current| {
                let prev = current.get();
                current.set(Some(self.0));
                prev
            });
            PREV.with(|prev| prev.set(prev_handle));
        }

        // Switch to `self`.
        platform::resume(self.0);

        // This is explicitly a different scope than before to avoid cross-contamination. We can't
        // make any assumptions about what was true before and after resuming another fiber, so we
        // enforce that we don't accidentally reuse any local variables. Only `PREV` and `CURRENT`
        // are safe to use.
        {
            let prev_fiber = PREV.with(|prev| prev.get().expect("PREV as None after resuming"));
            Fiber(prev_fiber)
        }
    }

    /// Retuns the fiber's unique ID.
    pub fn id(&self) -> FiberId {
        FiberId(self.0)
    }
}

// `Fiber` has pointers internally (at least on some platforms) so we need to manually implement
// `Send` and `Sync`. Sending should always be safe since fibers are designed to move between
// threads. The only thing potentially unsafe about sharing is that
unsafe impl Send for Fiber {}
unsafe impl Sync for Fiber {}

/// Returns the fiber that is currently executing on this thread.
///
/// Returns `None` if `init()` has not yet been called on this thread.
pub fn current() -> Option<FiberId> {
    CURRENT.with(|current| current.get()).map(|platform_fiber| FiberId(platform_fiber))
}
