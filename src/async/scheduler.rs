//! The main scheduler logic.
//!
//! The scheduler is implemented as a singleton in order to make it easy for code anywhere in the
//! project to make use of async functionality. The actual scheduler instance is not publicly
//! accessible, instead we use various standalone functions like `start()` and `wait_for()` to
//! safely manage access to the scheduler.

use fiber;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ptr::Unique;
use std::sync::{Condvar, Mutex, Once, ONCE_INIT};

pub use fiber::Fiber;

const DEFAULT_STACK_SIZE: usize = 64 * 1024;

static mut CONDVAR: *const Condvar = ::std::ptr::null();
static mut INSTANCE: *const Mutex<Scheduler> = ::std::ptr::null();
static INSTANCE_INIT: Once = ONCE_INIT;

thread_local! {
    /// Special fiber only used when needed to suspend a finished fiber.
    ///
    /// If a fiber finishes or needs to be supended while waiting on other fibers, it can't suspend
    /// unless there's another fiber for the thread to start executing. Normally this other fiber
    /// would be the next one in the work queue, but if there's no ready work then `WAIT_FIBER` is
    /// made active so that the previous fiber can be properly suspended.
    static WAIT_FIBER: Cell<Option<Fiber>> = Cell::new(None);

    /// Used to track which fiber was previously running after switching active fibers.
    ///
    /// The scheduler needs to track which fibers are active as it cannot try to make a fiber
    /// active on one thread if it's already active on another. It's not until a thread starts
    /// executing a fiber that it becomes safe to reuse or resume the previous fiber.
    static PREV_FIBER: Cell<Option<Fiber>> = Cell::new(None);
}

pub fn init_thread() {
    // Setup this thread for running fibers and create an initial fiber for it. This will become
    // the wait fiber for this thread.
    let fiber = fiber::init();
    // LOG: println!("initialized thread with fiber: {:?}", fiber);

    let fiber_proc = move || {
        // The current fiber has been resumed. Let the scheduler know that the previous fiber is no
        // longer active.
        let prev = PREV_FIBER.with(|prev| prev.get());
        Scheduler::with(|scheduler| {
            scheduler.handle_resumed(Fiber::current().unwrap(), prev);
        });

        loop {
            wait_for_work();
        }
    };

    let wait = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    // LOG: println!("Created wait fiber for thread: {:?}", wait);

    WAIT_FIBER.with(|wait_fiber| wait_fiber.set(Some(wait)));
    Scheduler::with(|scheduler| { scheduler.handle_resumed(fiber, None); });
}

pub fn run_wait_fiber() {
    // Setup this thread for running fibers and create an initial fiber for it. This will become
    // the wait fiber for this thread.
    let fiber = fiber::init();
    // LOG: println!("initialized thread with wait fiber: {:?}", fiber);

    WAIT_FIBER.with(|wait_fiber| { wait_fiber.set(Some(fiber)); });
    Scheduler::with(|scheduler| { scheduler.handle_resumed(fiber, None); });

    loop {
        wait_for_work();
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
        let prev = PREV_FIBER.with(|prev| prev.get());
        Scheduler::with(|scheduler| {
            scheduler.handle_resumed(Fiber::current().unwrap(), prev);
        });

        // Run the future, writing the result to `out`.
        *out_ptr.get_mut() = Some(func());

        // Finish the current fiber and run the next one.
        finish();
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    // LOG: println!("created fiber: {:?}", fiber);

    fiber
}

/// Schedules the provided future without suspending the current fiber.
pub fn run<F>(func: F) -> Fiber
    where
    F: FnOnce(),
    F: 'static + Send,
{
    let fiber_proc = move || {
        let prev = PREV_FIBER.with(|prev| prev.get());
        Scheduler::with(|scheduler| {
            scheduler.handle_resumed(Fiber::current().unwrap(), prev);
        });

        // Run the future, writing the result to `out`.
        func();

        // Finish the current fiber and run the next one.
        unsafe { finish(); }
    };

    let fiber = Fiber::new(
        DEFAULT_STACK_SIZE,
        fiber_proc,
    );

    // LOG: println!("created and starting fiber: {:?}", fiber);

    unsafe { start(fiber); }

    fiber
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
    // Retrieve the wait fiber for this thread.
    let wait_fiber = WAIT_FIBER.with(|wait| wait.get()).unwrap();
    let current = Fiber::current().unwrap();

    // Update the thread-local cache so that the next fiber can destroy this fiber or mark it as
    // suspended as necessary.
    PREV_FIBER.with(|prev| { prev.set(Fiber::current()); });

    // If we're on the the wait fiber, we are good to wait, using `CONDVAR` until work is
    // available. If we're on one of the normal work fibers then we can switch the next available
    // work unit, but if there isn't one then we need to immediately switch to the wait fiber so
    // that the current fiber can properly suspend.
    let next = if current == wait_fiber {
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

        next.unwrap()
    } else {
        if let Some(next) = Scheduler::with(|scheduler| scheduler.next()) {
            next
        } else {
            wait_fiber
        }
    };

    // LOG: println!("{:?} => {:?}", current, next);
    debug_assert!(next != current, "next {:?} cannot be current {:?}", next, current);
    next.make_active();

    // =============================== CROSSING THREAD LINES =============================== //
    // We are now potentially on a different thread, BE WARNED!



    // The current fiber has been resumed. Let the scheduler know that the previous fiber is no
    // longer active.
    let prev = PREV_FIBER.with(|prev| prev.get());
    Scheduler::with(|scheduler| {
        scheduler.handle_resumed(current, prev);
    });
}

pub struct Scheduler {
    running: HashSet<Fiber>,

    /// Fibers that have no pending dependencies.
    ///
    /// These are ready to be made active at any time.
    // TODO: This should be a queue, right?
    ready: Vec<Fiber>,

    /// A map specifying which pending fibers depend on which others.
    ///
    /// Once all of a fiber's dependencies complete it should be moved to `ready`.
    pending: HashMap<Fiber, HashSet<Fiber>>,

    finished: HashSet<Fiber>
}

impl Scheduler {
    /// Provides safe access to the scheduler instance.
    ///
    /// # Fiber Switches
    ///
    /// Note that it is an error to call `Fiber::make_active()` within `func`. Doing so will cause
    /// the `Mutex` guard on the instance to never unlock, making the scheduler instance
    /// inaccessible. All standalone functions that access the scheduler and wish to switch fibers
    /// should use `Scheduler::next()` to return the next fiber from `with()` and then call
    /// `make_active()` *after* `with()` has returned.
    fn with<F, T>(func: F) -> T
        where F: FnOnce(&mut Scheduler) -> T
    {
        INSTANCE_INIT.call_once(|| {
            let scheduler = Scheduler {
                running: HashSet::new(),
                ready: Vec::new(),
                pending: HashMap::new(),
                finished: HashSet::new(),
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
        debug_assert!(!self.running.contains(&fiber), "Can't schedule fiber {:?} which is already running", fiber);

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
        debug_assert!(!self.pending.contains_key(&pending), "Marking a fiber as pending but it is already pending: {:?}", pending);

        // Add `pending` to set of pending fibers and list `dependencies` as dependencies.
        let dependencies_set = HashSet::from_iter(dependencies.clone());
        self.pending.insert(pending, dependencies_set);

        // Add `fibers` to the list of ready fibers.
        for dependency in dependencies {
            // LOG: println!("making {:?} a dependency of {:?}", dependency, pending);
            if !self.running.contains(&dependency) {
                self.schedule(dependency);
            }
        }
    }

    /// Removes the specified fiber from the scheduler and updates dependents.
    fn finish(&mut self, fiber: Fiber) {
        // LOG: println!("finishing: {:?}", fiber);

        // Remove `fiber` as a dependency from other fibers, tracking any pending fibers that no
        // longer have any dependencies.
        let mut ready: Vec<Fiber> = Vec::new();
        for (pending, ref mut dependencies) in &mut self.pending {
            if dependencies.remove(&fiber) {
                // LOG: println!("removed {:?} as a dependency for {:?}", fiber, pending);
            }
            if dependencies.len() == 0 {
                // LOG: println!("{:?} has no dependencies, moving it to ready queue", pending);
                ready.push(*pending);
            }
        }

        // Remove any ready fibers from the pending set and add them to the ready set.
        for ready in ready {
            self.pending.remove(&ready);
            self.schedule(ready);
        }

        // Mark the fiber as complete. The actual work of updating dependencies won't happen until
        // the fiber has suspended, in `handle_suspended()`.
        self.finished.insert(fiber);
    }

    /// Performs the necessary bookkeeping when a fiber becomes active.
    fn handle_resumed(&mut self, resumed: Fiber, prev: Option<Fiber>) {
        // LOG: println!("resumed fiber: {:?}, prev: {:?}", resumed, prev);

        // Mark `resumed` as now being actively running.
        let was_not_running = self.running.insert(resumed.clone());
        debug_assert!(was_not_running, "Added {:?} as running but it was already running", resumed);

        // Handle `prev` if there was a previous fiber.
        if let Some(prev) = prev {
            let was_running = self.running.remove(&prev);
            debug_assert!(was_running, "Suspended a fiber that wasn't running: {:?}", prev);

            if self.finished.remove(&prev) {
                // TODO: Recycle fiber somehow.
            }
        }
    }

    /// Gets the next ready fiber and makes it active on the current thread.
    fn next(&mut self) -> Option<Fiber> {
        let popped = self.ready.pop();
        popped
    }
}
