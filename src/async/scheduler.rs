//! The main scheduler logic.
//!
//! The scheduler is implemented as a singleton in order to make it easy for code anywhere in the
//! project to make use of async functionality. The actual scheduler instance is not publicly
//! accessible, instead we use various standalone functions like `start()` and `wait_for()` to
//! safely manage access to the scheduler.
//!
//! # Scheduling Work
//!
//! Use `scheduler::start()` to run some work asynchronously, getting an `Async<T>` representing the
//! result of the work. Use `Async::await()` to suspend the current fiber until the work completes
//! and get the result. By default, dropping an `Async<T>` will suspend the current fiber until
//! the work finishes, but you can use `Async::forget()` to ignore the result without blocking.
//!
//! # Sharing Data Between Work
//!
//! Unlike with `std::thread::spawn()`, it's possible for work started with `scheduler::start()`
//! to borrow data from the caller:
//!
//! ```
//!
//! ```

use fiber::{self, Fiber, FiberId};
use cell_extras::AtomicInitCell;
use std::boxed::FnBox;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::sync::{Condvar, Mutex, Once, ONCE_INIT};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use stopwatch;

const DEFAULT_STACK_SIZE: usize = 64 * 1024;

static CONDVAR: AtomicInitCell<Condvar> = AtomicInitCell::new();
static INSTANCE: AtomicInitCell<Mutex<Scheduler>> = AtomicInitCell::new();
static INSTANCE_INIT: Once = ONCE_INIT;
static WORK_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Represents the result of a computation that may finish at some point in the future.
///
/// Use `scheduler::start()` to run some work asynchronously, getting an `Async<T>` representing the
/// result of the work. Use `Async::await()` to suspend the current fiber until the work completes
/// and get the result. By default, dropping an `Async<T>` will suspend the current fiber until
/// the work finishes, but you can use `Async::forget()` to ignore the result without blocking.
///
/// # Sharing Data Across Work
///
/// It's possible to share
#[derive(Debug)]
pub struct Async<'a, T> {
    work: WorkId,
    receiver: Receiver<T>,
    _phantom: PhantomData<&'a FnMut()>,
}

impl<'a, T> Async<'a, T> {
    /// Suspend the current fiber until the async operation finishes.
    pub fn await(self) -> T {
        let result = {
            let Async { work, ref receiver, .. } = self;
            work.await();
            receiver.try_recv().expect("Failed to receive result of async computation")
        };

        result
    }

    pub fn work_id(&self) -> WorkId {
        self.work
    }
}

impl<T> Async<'static, T> {
    pub fn forget(self) {
        mem::forget(self);
    }
}

impl<'a, T> Drop for Async<'a, T> {
    fn drop(&mut self) {
        self.work.await();
    }
}

/// A shareable reference to a work unit, counterpart to `Async<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkId(usize);

impl WorkId {
    /// Suspends the current fiber until this work unit has completed.
    ///
    /// If the work unit has already finished then `await()` will return immediately.
    pub fn await(self) {
        if Scheduler::with(|scheduler| scheduler.add_dependency(self)) {
            suspend();
        }
    }
}

/// Initializes a newly-spawned worker thread.
///
/// Prepares the worker thread by initializing it for Fiber usage.
// TODO: This should probably only be public within the crate. Only the engine should be using this,
// and only at startup, we probably don't want user code to be spawning threads anyway.
pub fn init_thread() {
    // Make sure the scheduler is initialized before first use.
    Scheduler::with(|_| {});

    // Manually convert current thread into a fiber because reasons.
    fiber::init();
}

// TODO: This should probably only be public within the crate. Only the engine should be using this,
// and only at startup, we probably don't want user code to be spawning threads anyway.
pub fn run_wait_fiber() {
    // Make sure the scheduler is initialized before first use.
    Scheduler::with(|_| {});

    // Setup this thread for running fibers and create an initial fiber for it. This will become
    // the wait fiber for this thread.
    fiber::init();

    fiber_routine();
}

pub fn start<'a, F, T>(func: F) -> Async<'a, T>
    where
    F: FnOnce() -> T,
    F: 'a + Send,
    T: 'a + Send,
{
    // Normally we can't box a closure with a non 'static lifetime because it could outlive its
    // borrowed data. In this case the lifetime parameter on the returned `Async` ensures that
    // the closure can't outlive the borrowed data, so we use this evil magic to convince the
    // compiler to allow us to box the closure.
    unsafe fn erase_lifetime<'a, F>(func: F) -> Box<FnBox()>
        where
        F: FnOnce(),
        F: 'a + Send,
    {
        let boxed_proc = Box::new(func);
        let proc_ptr = Box::into_raw(boxed_proc) as *mut FnBox();
        Box::from_raw(::std::mem::transmute(proc_ptr))
    }

    // Create the channel that'll be used to send the result of the operation to the `Async` object.
    let (sender, receiver) = mpsc::sync_channel(1);

    let work_id = WorkId(WORK_COUNTER.fetch_add(1, Ordering::Relaxed));

    let work_proc = unsafe {
        erase_lifetime(move || {
            let result = func();
            sender.try_send(result).expect("Failed to send async result");
        })
    };

    Scheduler::with(move |scheduler| scheduler.schedule_work(Work {
        func: work_proc,
        id: work_id,
    }));

    Async {
        work: work_id,
        receiver: receiver,
        _phantom: PhantomData,
    }
}

/// Suspends the current fiber and makes the wait fiber active.
///
/// Generally you shouldn't need to call this directly, but if you have one piece of code that
/// runs synchronously for a long time you can use `suspend()` to yield time to other work.
pub fn suspend() {
    let next_fiber = Scheduler::with(|scheduler| scheduler.next_fiber());

    let suspended = unsafe { next_fiber.resume() };

    stopwatch::switch_context(suspended.id(), fiber::current().unwrap());
    Scheduler::with(move |scheduler| scheduler.handle_suspended(suspended));
}

fn fiber_routine() -> ! {
    loop {
        match Scheduler::with(|scheduler| scheduler.next()) {
            Some(NextWork::Work(Work { func, id })) => {
                Scheduler::with(|scheduler| scheduler.start_work(id));
                func();
                Scheduler::with(|scheduler| scheduler.finish_work(id));
            },
            Some(NextWork::Fiber(fiber)) => {
                let suspended = unsafe { fiber.resume() };
                stopwatch::switch_context(suspended.id(), fiber::current().unwrap());
                Scheduler::with(move |scheduler| scheduler.handle_suspended(suspended));
            },
            None => {
                // If there's no new work and no fibers ready to run then we want to block the
                // thread until some becomes available.
                let mutex = INSTANCE.borrow();
                let condvar = CONDVAR.borrow();

                let _ = condvar
                    .wait(mutex.lock().expect("Scheduler mutex was poisoned"))
                    .expect("Scheduler mutex was poisoned");
            },
        }
    }
}

struct Work {
    func: Box<FnBox()>,
    id: WorkId,
}

impl Debug for Work {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "Work {{ id: {:?} }}", self.id)
    }
}

enum NextWork {
    Work(Work),
    Fiber(Fiber),
}

struct Scheduler {
    /// Work units that are currently pending or in progress.
    current_work: HashSet<WorkId>,

    work_map: HashMap<FiberId, WorkId>,

    /// New units of work that haven't been started on a fiber yet.
    ///
    /// These are ready to be made active at any time.
    new_work: VecDeque<Work>,

    /// Fibers that have no pending dependencies.
    ///
    /// These are ready to be made active at any time.
    ready_fibers: VecDeque<Fiber>,

    /// A map specifying which pending fibers depend on which others.
    ///
    /// Once all of a fiber's dependencies complete it should be moved to `new_work`.
    dependencies: HashMap<FiberId, (Option<Fiber>, HashSet<WorkId>)>,

    // TODO: Should we distinguise between "finished" and "ready" fibers? My intuition is that we'd
    // want to give fibers that actively have work CPU time before we resume fibers that would be
    // pulling new work, but maybe not? If we threw them all into one queue I guess the worst case
    // scenario would be there's no work left, a bunch of empty fibers, and only a few fibers with
    // active work. In which case we might have to cycle through a bunch of fibers before we can
    // start doing actual work.
    finished: VecDeque<Fiber>,
}

unsafe impl Send for Scheduler {}

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
                current_work: HashSet::new(),
                work_map: HashMap::new(),
                new_work: VecDeque::new(),
                ready_fibers: VecDeque::new(),
                dependencies: HashMap::new(),
                finished: VecDeque::new(),
            };

            INSTANCE.init(Mutex::new(scheduler));
            CONDVAR.init(Condvar::new());
        });

        let instance = INSTANCE.borrow();

        let mut guard = instance.lock().expect("Scheduler mutex was poisoned");
        func(&mut *guard)
    }

    /// Add a new unit of work to the pending queue.
    fn schedule_work(&mut self, work: Work) {
        assert!(self.current_work.insert(work.id), "Work's ID was already present in current work set");
        self.new_work.push_back(work);
        CONDVAR.borrow().notify_one();
    }

    /// Adds `dependency` as a dependency of the currently running fiber.
    ///
    /// Returns `true` if work is still in progress and was added as a dependency, false otherwise.
    fn add_dependency(&mut self, dependency: WorkId) -> bool {
        let pending = fiber::current().unwrap();

        if self.current_work.contains(&dependency) {
            debug_assert!(
                !self.dependencies.contains_key(&pending),
                "Marking a fiber as pending but it is already pending: {:?}",
                pending,
            );

            // Add `pending` to set of pending fibers and list `dependencies` as dependencies.
            let &mut (_, ref mut dependencies_set) =
            self.dependencies
            .entry(pending)
            .or_insert((None, HashSet::new()));

            // Add `fibers` to the list of ready fibers.
            dependencies_set.insert(dependency);

            true
        } else {
            false
        }
    }

    fn start_work(&mut self, new_work: WorkId) {
        debug_assert!(self.current_work.contains(&new_work), "Work ID was not in current work set");

        let current = fiber::current().unwrap();
        self.work_map.insert(current, new_work);
    }

    /// Removes the specified unit of work from the scheduler, updating any dependent work.
    fn finish_work(&mut self, finished_work: WorkId) {
        // Iterate over all suspended work units, removing `finished_work` as a dependency where
        // necessary. If any of the work units no longer have dependencies then
        let mut ready = Vec::new();
        for (&pending_fiber, &mut (_, ref mut dependencies)) in &mut self.dependencies {
            dependencies.remove(&finished_work);
            if dependencies.len() == 0 {
                ready.push(pending_fiber);
            }
        }

        for ready_work in ready {
            let (maybe_fiber, _) = self.dependencies.remove(&ready_work).unwrap();
            if let Some(ready_fiber) = maybe_fiber {
                self.ready_fibers.push_back(ready_fiber);
                CONDVAR.borrow().notify_one();
            }
        }

        let fiber = fiber::current().unwrap();
        assert!(self.current_work.remove(&finished_work), "{:?} wasn't in current work set when it finished", finished_work);
        assert!(self.work_map.remove(&fiber).is_some(), "{:?} didn't have {:?} associated in the work map", fiber, finished_work);
    }

    /// Performs the necessary bookkeeping when a fiber becomes active.
    fn handle_suspended(&mut self, suspended: Fiber) {
        // If the suspended fiber has dependencies then update the dependencies map with the
        // actual fiber, that way when its dependencies complete it can be resumed. Otherwise, the
        // fiber is done and ready to take on more work. This means that we need to make sure that
        // we always call `add_dependencies()` before suspending a fiber, otherwise a fiber could
        // be marked as done before it's ready.
        if let Some(&mut (ref mut none_fiber, _)) = self.dependencies.get_mut(&suspended.id()) {
            debug_assert!(none_fiber.is_none(), "Dependencies map already had a fiber assicated with fiber ID");
            mem::replace(none_fiber, Some(suspended));
        } else if self.work_map.contains_key(&suspended.id()) {
            self.ready_fibers.push_back(suspended);
        } else {
            self.finished.push_back(suspended);
        }
    }

    /// Gets the next ready fiber, or creates a new one if necessary.
    fn next_fiber(&mut self) -> Fiber {
        self.ready_fibers.pop_front()
            .or_else(|| self.finished.pop_front())
            .unwrap_or_else(|| {
                fn fiber_proc(suspended: Fiber) -> ! {
                    stopwatch::switch_context(suspended.id(), fiber::current().unwrap());

                    // The current fiber has been resumed. Let the scheduler know that the previous fiber is no
                    // longer active.
                    Scheduler::with(|scheduler| scheduler.handle_suspended(suspended));

                    fiber_routine();
                }

                Fiber::new(DEFAULT_STACK_SIZE, fiber_proc)
            })
    }

    /// Gets the next available work for a thread, either a new unit of work or a ready fiber.
    ///
    /// Prioritizes new work over pending fibers, and will only return ready fibers that already
    /// have work. To get *any* next fiber, including ones without active work or a new one if no
    /// existing fibers are available, use `next_fiber()`.
    fn next(&mut self) -> Option<NextWork> {
        if let Some(work) = self.new_work.pop_front() {
            Some(NextWork::Work(work))
        } else {
            self.ready_fibers.pop_front().map(|fiber| NextWork::Fiber(fiber))
        }
    }
}
