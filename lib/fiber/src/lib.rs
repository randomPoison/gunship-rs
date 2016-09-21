//! A libray for creating and managing fibers in a cross-platform manner.
//!
//! Fibers are threads that must be manually scheduled by the client application, as opposed to
//! threads which are automatically managed and scheduled by the OS. Each fiber has its own stack
//! space and can be yield its time to another thread at any point during execution. This allows
//! for different forms of concurrency to be implemented in a way that's optimal for the client
//! application.

#[cfg(target_os="windows")]
#[path="platform\\windows.rs"]
pub mod platform;

pub type FiberProc<T> = extern "system" fn (*mut T);

#[derive(Debug)]
pub struct Fiber(platform::Fiber);

pub fn init() -> Fiber {
    let fiber = platform::init();
    Fiber(fiber)
}

impl Fiber {
    /// Creates a new fiber with the specified stack size and has it begin executing the specified
    /// function.
    pub fn new<T>(stack_size: usize, function: FiberProc<T>, data: *mut T) -> Fiber {
        let fiber = platform::create_fiber(
            stack_size,
            function,
            data,
        );
        Fiber(fiber)
    }

    /// Makes the fiber active, suspending the current one.
    pub fn make_active(&self) {
        platform::make_active(self.0);
    }
}
