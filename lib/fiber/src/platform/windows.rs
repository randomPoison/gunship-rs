extern crate kernel32;
extern crate winapi;

use FiberProc;
use std::ptr;
use self::winapi::*;

pub type Fiber = LPVOID;

pub fn init() -> Fiber {
    let fiber = unsafe { kernel32::ConvertThreadToFiber(ptr::null_mut()) };

    if fiber.is_null() {
        println!("ERROR: Failed to convert main thread to a fiber");
    } else {
        println!("initialized main fiber: {:?}", fiber);
    }

    fiber
}

pub fn create_fiber<T>(stack_size: usize, fiber_proc: FiberProc<T>, data: *mut T) -> Fiber {
    let fiber = unsafe {
        let platform_proc = ::std::mem::transmute(fiber_proc);
        kernel32::CreateFiber(
            stack_size as u32,
            Some(platform_proc),
            data as LPVOID,
        )
    };

    // TODO: Return an error result, rather than just logging a warning.
    if fiber.is_null() {
        println!("ERROR: Failed to create fiber");
    } else {
        println!("created fiber: {:?}", fiber);
    }

    fiber
}

pub fn make_active(fiber: Fiber) {
    println!("making active: {:?}", fiber);
    unsafe { kernel32::SwitchToFiber(fiber); }
}
