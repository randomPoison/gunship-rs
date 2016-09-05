//! The main scheduler logic.
//!
//! The scheduler is implemented as a singleton in order to make it easy for code anywhere in the
//! project to make use of async functionality. The actual scheduler instance is not publicly
//! accessible, instead we use various standalone functions like `start()` and `wait_for()` to
//! safely manage access to the scheduler.

use fiber::{self, Fiber};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::{Condvar, Mutex, Once, ONCE_INIT};

static mut CONDVAR: *const Condvar = ::std::ptr::null();
static mut INSTANCE: *const Mutex<Scheduler> = ::std::ptr::null();
static INSTANCE_INIT: Once = ONCE_INIT;

/// Schedules `fiber` without suspending the current fiber.
pub fn start(fiber: Fiber) {
    Scheduler::with(move |scheduler| {
        scheduler.schedule(fiber);
    });
}

/// Suspends the current fiber until `fiber` completes.
pub fn wait_for(fiber: Fiber) {
    Scheduler::with(move |scheduler| {
        let current = Fiber::current().expect("Unable to get current fiber");
        scheduler.wait_for(current, fiber);
    });

    wait_for_work();
}

/// Suspends the current fiber until all fibers in `fibers` complete.
pub fn wait_for_all<I>(fibers: I)
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
pub fn finish() {
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
