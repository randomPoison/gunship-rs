//! The main scheduler logic.
//!
//! The scheduler is implemented as a singleton in order to make it easy for code anywhere in the
//! project to make use of async functionality. The actual scheduler instance is not publicly
//! accessible, instead we use various standalone functions like `start()` and `wait_for()` to
//! safely manage access to the scheduler.

use fiber::{self, Fiber};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ptr::Unique;
use std::sync::{Condvar, Mutex, Once, ONCE_INIT};

static mut CONDVAR: *const Condvar = ::std::ptr::null();
static mut INSTANCE: *const Mutex<Scheduler> = ::std::ptr::null();
static INSTANCE_INIT: Once = ONCE_INIT;

const DEFAULT_STACK_SIZE: usize = 64 * 1024;

pub fn init() {
    // No-op invocation of `with()` to force initialization. Honestly this is kind of dumb, we
    // don't need lazy initialization if we're explicitly initializing on startup.
    Scheduler::with(|_| {});
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
        finish();
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
        unsafe { finish(); }
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    unsafe { start(fiber); }
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
    unsafe {
        let fiber = create_fiber(func, out);
        wait_for(fiber);
    }
}

/// Suspends the current fiber until all fibers in `fibers` complete.
///
/// It's generally not advisable to call `await_all()` directly, instead use the `await_all!()`
/// macro, which handles the work of converting futures into fibers and returning their results
/// after.
pub unsafe fn await_all<I: IntoIterator<Item = Fiber> + Clone>(fibers: I) {
    wait_for_all(fibers);
}


/// Schedules `fiber` without suspending the current fiber.
pub unsafe fn start(fiber: Fiber) {
    Scheduler::with(move |scheduler| {
        scheduler.schedule(fiber);
    });
}

/// Suspends the current fiber until `fiber` completes.
pub unsafe fn wait_for(fiber: Fiber) {
    Scheduler::with(move |scheduler| {
        let current = Fiber::current().expect("Unable to get current fiber");
        scheduler.wait_for(current, fiber);
    });

    wait_for_work();
}

/// Suspends the current fiber until all fibers in `fibers` complete.
pub unsafe fn wait_for_all<I>(fibers: I)
    where
    I: IntoIterator<Item = Fiber>,
    I: Clone,
{
    Scheduler::with(move |scheduler| {
        let current = Fiber::current().expect("Unable to get current fiber");
        scheduler.wait_for_all(current, fibers);
    });

    wait_for_work();
}

/// Ends the current fiber and begins the next ready one.
pub unsafe fn finish() {
    Scheduler::with(|scheduler| {
        let current = Fiber::current().expect("Unable to get current fiber");
        scheduler.finish(current);
    });

    wait_for_work();
}

/// Suspends the current thread until the scheduler has more work available.
pub fn wait_for_work() {
    let mut next;
    unsafe {
        let mutex = &*INSTANCE;
        let condvar = &*CONDVAR;

        let mut scheduler = mutex.lock().expect("Scheduler mutex was poisoned");
        next = scheduler.next();
        while next.is_none() {
            scheduler =
                condvar
                .wait(scheduler)
                .expect("Scheduler mutex was poisoned");
            next = scheduler.next();
        }
    }

    // Once
    next.unwrap().make_active();
}

pub struct Scheduler {
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
    /// Provides safe access to the scheduler instance.
    ///
    /// # Fiber Switches
    ///
    /// Note that it is an error to call `Fiber::make_active()` within `func`. Doing so will cause
    /// the `Mutex` guard on the instance to never unlock, making the scheduler instance
    /// inaccessible. All standalone functions that access the sceduler and wish to switch fibers
    /// should use `Scheduler::next()` to return the next fiber from `with()` and then call
    /// `make_active()` *after* `with()` has returned.
    pub fn with<F, T>(func: F) -> T
        where F: FnOnce(&mut Scheduler) -> T
    {
        INSTANCE_INIT.call_once(|| {
            fiber::init();

            let scheduler = Scheduler {
                ready: Vec::new(),
                pending: HashMap::new(),
                finished: Vec::new(),
            };

            let boxed_scheduler = Box::new(Mutex::new(scheduler));
            unsafe { INSTANCE = Box::into_raw(boxed_scheduler); }

            let boxed_condvar = Box::new(Condvar::new());
            unsafe { CONDVAR = Box::into_raw(boxed_condvar); }
        });

        let mutex = unsafe {
            assert!(!INSTANCE.is_null(), "Scheduler instance is null");
            &*INSTANCE
        };
        let mut guard = mutex.lock().expect("Scheduler mutex was poisoned");
        func(&mut *guard)
    }

    /// Schedules `fiber` without any dependencies;
    fn schedule(&mut self, fiber: Fiber) {
        self.ready.push(fiber);

        let condvar = unsafe { &*CONDVAR };
        condvar.notify_one();
    }

    /// Schedules `dependency` and suspends `pending` until it finishes.
    fn wait_for(&mut self, pending: Fiber, dependency: Fiber) {
        self.wait_for_all(pending, [dependency].iter().cloned());
    }

    /// Schedules the current fiber as pending, with dependencies on `fibers`.
    fn wait_for_all<I>(&mut self, pending: Fiber, dependencies: I)
        where
        I: IntoIterator<Item = Fiber>,
        I: Clone,
    {
        // Add `pending` to set of pending fibers and list `dependencies` as dependencies.
        let dependencies_set = HashSet::from_iter(dependencies.clone());
        self.pending.insert(pending, dependencies_set);

        // Add `fibers` to the list of ready fibers.
        for dependency in dependencies {
            self.schedule(dependency);
        }
    }

    /// Removes the specified fiber from the scheduler and updates dependents.
    fn finish(&mut self, done: Fiber) {
        // Remove `done` as a dependency from other fibers, tracking any pending fibers that no
        // longer have any dependencies.
        let mut ready: Vec<Fiber> = Vec::new();
        for (fiber, ref mut dependencies) in &mut self.pending {
            if dependencies.remove(&done) && dependencies.len() == 0 {
                ready.push(fiber.clone());
            }
        }

        // Remove any ready fibers from the pending set and add them to the ready set.
        for fiber in &ready {
            self.pending.remove(fiber);
        }
        self.ready.extend(ready);

        // Mark the done fiber as complete.
        // TODO: This is wrong, another thread may attempt to free this fiber before it is suspended.
        self.finished.push(done);
    }

    /// Gets the next ready fiber and makes it active on the current thread.
    fn next(&mut self) -> Option<Fiber> {
        let popped = self.ready.pop();
        popped
    }
}
