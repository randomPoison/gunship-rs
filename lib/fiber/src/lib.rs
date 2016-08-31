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

#![feature(fnbox)]

use std::boxed::FnBox;
use std::cell::RefCell;

#[cfg(target_os="windows")]
#[path="platform\\windows.rs"]
pub mod platform;

/// Represents a fiber with its own stack and thread state.
///
/// The fiber's lifetime is associated with any data it borrows when created.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fiber {
    inner: platform::Fiber,
}

/// A global cache mapping threads to their currently running fiber.
///
/// This is used by `Fiber::current()` on some platforms to keep track of which fiber is active
/// on which thread.
thread_local!{
    static CURRENT: RefCell<Option<Fiber>> = RefCell::new(None);
}

/// Initializes the current thread, making it safe to begin using threads.
///
/// On some platforms initialization is required before using threads (e.g. on Windows versions
/// older than 7 the main thread must be converted to a fiber on startup). This method performs
/// any necessary initialization and returns the active fiber. This method must be called for all
/// spawned threads to ensure that it is safe to use fibers from those threads.
pub fn init() -> Fiber {
    // Perform platform-specific initialization.
    let fiber = Fiber {
        inner: platform::init(),
    };

    // Initialize our thread-local cache of the current fiber.
    CURRENT.with(|current| *current.borrow_mut() = Some(fiber.clone()));

    fiber
}

impl Fiber {
    /// Creates a new fiber with the specified stack size and has it begin executing the specified
    /// function.
    ///
    /// # Panics
    ///
    /// Panics if `init()` has not yet been called on the current thread.
    // TODO: Allow fiber proc to return a value? Or should we continue to handle that in client code?
    pub fn new<F>(stack_size: usize, func: F) -> Fiber
        where
        F: 'static,
        F: FnOnce(),
        F: Send,
    {
        // Ensure that we've already been initialized and can safely start using fibers.
        assert!(Fiber::current().is_some(), "Fibers not initialized, call `init()` before `Fiber::new()`");

        let fiber = platform::create_fiber(
            stack_size,
            Box::new(func) as Box<FnBox()>,
        );
        Fiber {
            inner: fiber,
        }
    }

    /// Returns the fiber that is currently executing on this thread.
    ///
    /// Returns `None` if `init()` has not yet been called on this thread.
    pub fn current() -> Option<Fiber> {
        CURRENT.with(|current| current.borrow().clone())
    }

    /// Makes the fiber active, suspending the current one.
    pub fn make_active(&self) {
        // Update thread-local cache to track `self` as active.
        CURRENT.with(|current| *current.borrow_mut() = Some(self.clone()));

        // Switch to `self`.
        platform::make_active(self.inner);
    }
}
