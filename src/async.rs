//! Provides mangement and scheduling for fiber-based tasks in the engine.
//!
//! This module acts as a singleton. This is to allow the scheduler to globally accessible, making
//! async operations usable from anywhere in the engine and game code.

// extern crate futures;

use fiber::{self, Fiber};
static mut INSTANCE: Option<*mut Scheduler> = None;

pub trait Future {
    type Item;
    type Error;

    fn run(&mut self) -> Result<Self::Item, Self::Error>;
}

trait FiberWork : Future {
    /// Writes the result of `wait()` to `out`.
    ///
    /// This is used to make type-erasure on futures safe.
    fn wait(&mut self, out: &mut Option<Result<Self::Item, Self::Error>>) {
        *out = Some(self.run());
    }
}

impl<T: Future> FiberWork for T {}

/// Creates the scheduler instance, enabling async task execution.
pub fn init() {
    let current = fiber::init();

    let scheduler = Scheduler {
        stack: vec![current],
    };

    let boxed_scheduler = Box::new(scheduler);
    unsafe { INSTANCE = Some(Box::into_raw(boxed_scheduler)); }
}

/// Suspends the current fiber until the specified future completes.
///
/// The result of the provided fiber will be written to `out`.
///
/// TODO: What happens if `future` crashes or never completes?
///
/// # Unsafety
///
/// `out` must still be a safe output location for the result of `future` when it completes. It is
/// meant to be an address on the stack of the current fiber, but an address on the heap will work
/// as long as it's owned by the current fiber.
pub unsafe fn run_async<F: 'static + Future>(future: F, out: &mut Option<Result<F::Item, F::Error>>) {
    let mut fiber_data = FiberData {
        future: Box::new(future),
        out: out,
    };

    println!("making work fiber");
    let fiber = Fiber::new(
        2 * 1024 * 1024,
        fiber_proc,
        &mut fiber_data as *mut FiberData<F::Item, F::Error> as *mut FiberData<_, _>,
    );

    Scheduler::push(fiber);
}

/// Loads a pending unit of work off the work queue and runs it to completion.
///
/// This is the entry point for all work fibers.
extern "system" fn fiber_proc(data: *mut FiberData<(), ()>) {
    println!("fiber_proc({:?})", data);


    unsafe {
        let data = &mut *data;
        data.future.wait(&mut *data.out);
    }

    Scheduler::pop();
}

struct FiberData<F, E> {
    future: Box<FiberWork<Item=F, Error=E>>,
    out: *mut Option<Result<F, E>>,
}

/// TODO: We need synchronoziation around the scheduler internals.
struct Scheduler {
    stack: Vec<Fiber>,
}

impl Scheduler {
    fn push(fiber: Fiber) {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };
        scheduler.stack.push(fiber);

        let fiber = scheduler.stack.last().unwrap();

        println!("making work fiber active");
        fiber.make_active();
        println!("work fiber finished");
    }

    fn pop() {
        let scheduler = unsafe { &mut *INSTANCE.expect("Scheduler does not exist") };
        scheduler.stack.pop();
        if let Some(fiber) = scheduler.stack.last() {
            fiber.make_active();
        }
    }
}
