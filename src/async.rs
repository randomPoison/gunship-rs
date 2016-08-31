//! Provides mangement and scheduling for fiber-based tasks in the engine.
//!
//! This module acts as a singleton. This is to allow the scheduler to globally accessible, making
//! async operations usable from anywhere in the engine and game code.

use fiber::{self, Fiber};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ptr::Unique;

const DEFAULT_STACK_SIZE: usize = 2 * 1024 * 1024;

pub trait Future: 'static + Send {
    type Item: 'static + Send;
    type Error: 'static + Send;

    fn run(&mut self) -> Result<Self::Item, Self::Error>;
}

/// Creates the scheduler instance, enabling async task execution.
// TODO: Don't expose outside of the crate, it should only be used internally to the engine.
pub fn init() {
    fiber::init();

    let scheduler = Scheduler {
        ready: Vec::new(),
        pending: HashMap::new(),
        finished: Vec::new(),
    };

    let boxed_scheduler = Box::new(scheduler);
    unsafe { INSTANCE = Some(Box::into_raw(boxed_scheduler)); }
}

/// Schedules a fiber without suspending the current one.
///
/// # Unsafety
///
/// `out` must live long enough that the fiber can still write its result when it completes, or it
/// will write to invalid memory. Unlike `await()` this function doens't suspend the current fiber
/// so any code calling `start()` must ensure that it suspends the current fiber until `future` has
/// had a chance to write to `out`.
pub unsafe fn start<F>(
    mut future: F,
    out: &mut Option<Result<F::Item, F::Error>>,
) -> Fiber
    where F: 'static + Future
{
    // `*mut _` isn't `Send` (for good reason), so we need to assure the compiler that we know what
    // we're doing. `Unique` specifies that a `*mut _` isn't shared, so it's safe(-er) to send
    // between threads.
    let mut out_ptr = Unique::new(out as *mut _);

    let fiber_proc = move || {
        // Run the future, writing the result to `out`.
        *out_ptr.get_mut() = Some(future.run());
        Scheduler::finish();
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    Scheduler::schedule(fiber.clone());

    fiber
}

/// Suspends the current fiber until the specified future completes.
///
/// The result of the provided fiber will be written to `out`.
// TODO: What happens if `future` crashes or never completes?
pub fn await<F: Future>(mut future: F, out: &mut Option<Result<F::Item, F::Error>>) {
    // `*mut _` isn't `Send` (for good reason), so we need to assure the compiler that we know what
    // we're doing. `Unique` specifies that a `*mut _` isn't shared, so it's safe(-er) to send
    // between threads.
    let mut out_ptr = unsafe { Unique::new(out as *mut _) };

    let fiber_proc = move || {
        // Run the future, writing the result to `out`.
        unsafe { *out_ptr.get_mut() = Some(future.run()); }
        Scheduler::finish();
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    Scheduler::wait_for(fiber);
}

pub fn await_all<I: IntoIterator<Item = Fiber> + Clone>(fibers: I) {
    Scheduler::wait_for_all(fibers);
}

// TODO: We need synchronization around either the scheduler internals or the instance.
static mut INSTANCE: Option<*mut Scheduler> = None;

struct Scheduler {
    /// Fibers that have no pending dependencies.
    ///
    /// These are ready to be made active at any time.
    // TODO: This should be a queue, right?
    ready: Vec<Fiber>,

    /// A map specifying which pending fibers depend on which others.
    ///
    /// Once all of a fiber's dependencies complete it should be moved to `ready`.
    pending: HashMap<Fiber, HashSet<Fiber>>,

    /// Fibers that have finished their work and can be freed.
    finished: Vec<Fiber>,
}

impl Scheduler {
    /// Suspends the current fiber until `fiber` completes.
    fn wait_for(fiber: Fiber) {
        Scheduler::wait_for_all([fiber].iter().cloned());
    }

    /// Suspends the current fiber until all fibers in `fibers` complete.
    fn wait_for_all<I>(fibers: I)
        where
        I: IntoIterator<Item = Fiber>,
        I: Clone,
    {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };
        let current = Fiber::current().expect("BUG: `Fiber::current()` returned `None`");

        // Add `current` to set of pending fibers and list `fibers` as dependencies.
        let dependencies = HashSet::from_iter(fibers.clone());
        scheduler.pending.insert(current, dependencies);

        // Add `fibers` to the list of ready fibers.
        scheduler.ready.extend(fibers);

        // Suspend the current fiber and resume the next available one.
        Scheduler::run_next();
    }

    /// Schedules `fiber` without suspending the current fiber.
    fn schedule(fiber: Fiber) {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };
        scheduler.ready.push(fiber);
    }

    /// Removes the current fiber from the scheduler and resumes the next ready fiber.
    fn finish() {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };
        let current = Fiber::current().expect("BUG: `Fiber::current()` returned `None`");

        // Remove `current` as a dependency from other fibers, tracking any pending fibers that no
        // longer have any dependencies.
        let mut ready: Vec<Fiber> = Vec::new();
        for (fiber, ref mut dependencies) in &mut scheduler.pending {
            if dependencies.remove(&current) && dependencies.len() == 0 {
                ready.push(fiber.clone());
            }
        }

        // Remove any ready fibers from the pending set and add them to the ready set.
        for fiber in &ready {
            scheduler.pending.remove(fiber);
        }
        scheduler.ready.extend(ready);

        // Mark the current fiber as complete.
        // TODO: This is wrong, another thread may attempt to free this fiber before it is suspended.
        scheduler.finished.push(current);

        // Suspend the current fiber and resume the next available one.
        Scheduler::run_next();
    }

    /// Gets the next ready fiber and makes it active on the current thread.
    fn run_next() {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };

        match scheduler.ready.pop() {
            Some(fiber) => fiber.make_active(),

            // TODO: Suspend until work is available.
            None => println!("WARNING: No more work to do, this thread is going to hang"),
        }
    }
}
