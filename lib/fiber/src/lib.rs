//! A libray for creating and managing fibers in a cross-platform manner.
//!
//! Fibers are threads that must be manually scheduled by the client application, as opposed to
//! threads which are automatically managed and scheduled by the OS. Each fiber has its own stack
//! space and can be yield its time on the system thread to another fiber at any point during
//! execution. This allows for different forms of concurrency that can't be supported with
//! normal system threads.
//!
//! This library is meant to be the base for a fiber-pool system, in which a fixed number of worker
//! fibers are created and used to asynchronously complete units of work.
//!
//! # Fibers and Threads
//!
//! Fibers are run on top of system threads, with one fiber running on a thread at a time. Once
//! a fiber has been suspended it can be resumed on any thread (i.e the same thread it was
//! previously on or any other thread). You don't have to have multiple threads to use fibers,
//! but if only using a single thread fibers won't run in parallel. As such, it's generally best
//! to use fibers in combination with a pool of worker threads.
//!
//! Being able to move a fiber between threads also has implications for the thread-safety of
//! your code. There are a number of system-primitives that don't take well to moving between
//! threads, and so you must be careful when you resume fibers. Notably, [`Mutex`][mutex]
//!
//! # Unsafety
//!
//! Unlike any other function in Rust, fiber procs can be suspended on one thread and resumed
//! on another, pulling any stack-owned data along with it. That means that it's possible to
//! create a `!Send` type, suspend the fiber, and resume the fiber on another thread,
//! violating the `!Send` nature of the type. As such, it's unsafe to ever call `Fiber::resume()`
//! while a `!Send` type is alive and in scope.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use fiber::Fiber;
//!
//! // Function to be run by the fiber. Return type must be `!`, this indicates that it will never
//! // return. This is necessary because fibers cannot be destroyed on some platforms.
//! fn fiber_proc(suspended: Fiber) -> ! {
//!     println!("Suspended fiber: {:?}", suspended);
//!     unsafe { suspended.resume(); }
//!
//!     panic!("Uh-oh, shouldn't have resumed this fiber again");
//! }
//!
//! let fiber = Fiber::new(1024, fiber_proc);
//! let fiber_id = fiber.id();
//!
//! let prev = unsafe { fiber.resume() };
//! assert_eq!(fiber_id, prev.id());
//! ```

use platform::PlatformId;
use std::cell::Cell;

#[cfg(target_os="windows")]
#[path="platform\\windows.rs"]
pub mod platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiberId(PlatformId);

impl FiberId {
    // HACK: This is used only in stopwatch to serialze the value in JSON. We should probably
    // support this usecase in a more direct manner.
    pub fn primitive_id(self) -> isize {
        self.0 as isize
    }
}

// `FiberId` contains a raw pointer (at least on some platforms) so it's not `Send`/`Sync` by
// default, but it can't actually be used for anything unsafe so we manually confirm that it can
// be shared and sent between threads.
unsafe impl Send for FiberId {}
unsafe impl Sync for FiberId {}

/// Represents a fiber with its own stack and thread state.
///
/// The fiber's lifetime is associated with any data it borrows when created.
// TODO: What do we do about `Drop`? If you drop a `Fiber` it's basically leaking a sizeable chunk
// of memory. Should we emit a warning or panic?
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
// TODO: How do we handle double-initialization? Panic or just ignore it?
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
    /// # Fiber Proc
    ///
    /// TODO: Talk about fiber proc and why it can't return.
    // TODO: Should we return a `Result` here? Do we want to allow thread creation to fail? That'd
    // kind of be like the equivalent of OOM when requesting a heap allocation, in which case
    // I think the stdlib panics.
    pub fn new(stack_size: usize, fiber_proc: fn(Fiber) -> !) -> Fiber
    {
        Fiber(platform::create_fiber(stack_size, fiber_proc))
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
// threads. The only thing potentially unsafe about sharing would be trying to resume a fiber
// on two different threads, but the signature of `Fiber::resume()` statically prevents that.
unsafe impl Send for Fiber {}
unsafe impl Sync for Fiber {}

/// Returns the fiber that is currently executing on this thread.
///
/// Returns `None` if `init()` has not yet been called on this thread.
pub fn current() -> Option<FiberId> {
    CURRENT.with(|current| current.get()).map(|platform_fiber| FiberId(platform_fiber))
}
