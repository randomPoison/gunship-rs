//! Provides mangement and scheduling for fiber-based tasks in the engine.
//!
//! This module acts as a singleton. This is to allow the scheduler to globally accessible, making
//! async operations usable from anywhere in the engine and game code.

use fiber::{self, Fiber};
use std::ptr::Unique;

mod scheduler;

const DEFAULT_STACK_SIZE: usize = 64 * 1024;

pub fn init() {
    // No-op invocation of `with()` to force initialization. Honestly this is kind of dumb, we
    // don't need lazy initialization if we're explicitly initializing on startup.
    scheduler::Scheduler::with(|_| {});
}

pub fn start_workers(worker_count: usize) {
    for _ in 0..worker_count {
        ::std::thread::spawn(|| {
            // Initialize worker thread for fibers.
            fiber::init();

            // Wait until work is available for this thread.
            scheduler::wait_for_work();
        });
    }
}

/// Creates a fiber from the given function.
///
/// # Unsafety
///
/// `out` must live long enough that the fiber can still write its result when it completes, or it
/// will write to invalid memory. Unlike `await()` this function doens't suspend the current fiber
/// so any code calling `create_fiber()` must ensure that it suspends the current fiber until `func` has
/// had a chance to write to `out`.
///
/// Working with fibers directly is inherently unsafe as making a fiber active at the wrong time
/// could leave an operation in an undefined state.
pub unsafe fn create_fiber<F, I, E>(
    func: F,
    out: &mut Option<Result<I, E>>,
) -> Fiber
    where
    F: FnOnce() -> Result<I, E>,
    F: 'static + Send,
    I: 'static + Send,
    E: 'static + Send,
{
    // `*mut _` isn't `Send` (for good reason), so we need to assure the compiler that we know what
    // we're doing. `Unique` specifies that a `*mut _` isn't shared, so it's safe(-er) to send
    // between threads.
    let mut out_ptr = Unique::new(out as *mut _);

    let fiber_proc = move || {
        // Run the future, writing the result to `out`.
        *out_ptr.get_mut() = Some(func());

        // Finish the current fiber and run the next one.
        scheduler::finish();
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    fiber
}

/// Schedules the provided future without suspending the current fiber.
pub fn run<F>(func: F)
    where
    F: FnOnce(),
    F: 'static + Send,
{
    let fiber_proc = move || {
        // Run the future, writing the result to `out`.
        func();

        // Finish the current fiber and run the next one.
        scheduler::finish();
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    scheduler::start(fiber);
}

/// Suspends the current fiber until the specified future completes.
///
/// The result of the provided fiber will be written to `out`. It's generally not advisable to
/// call `await()` directly, instead use the `await!()` macro which returns the result directly.
// TODO: What happens if `func` crashes or never completes?
pub fn await<F, I, E>(func: F, out: &mut Option<Result<I, E>>)
    where
    F: FnOnce() -> Result<I, E>,
    F: 'static + Send,
    I: 'static + Send,
    E: 'static + Send,
{
    let fiber = unsafe { create_fiber(func, out) };
    scheduler::wait_for(fiber);
}

/// Suspends the current fiber until all fibers in `fibers` complete.
///
/// It's generally not advisable to call `await_all()` directly, instead use the `await_all!()`
/// macro, which handles the work of converting futures into fibers and returning their results
/// after.
pub fn await_all<I: IntoIterator<Item = Fiber> + Clone>(fibers: I) {
    scheduler::wait_for_all(fibers);
}
