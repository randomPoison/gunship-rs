extern crate kernel32;
extern crate winapi;

use std::boxed::FnBox;
use std::ptr;
use self::winapi::*;

pub type Fiber = LPVOID;

pub fn init() -> Fiber {
    let fiber = unsafe { kernel32::ConvertThreadToFiber(ptr::null_mut()) };

    if fiber.is_null() {
        println!("ERROR: Failed to convert main thread to a fiber");
    }

    fiber
}

pub fn create_fiber(stack_size: usize, func: Box<FnBox()>) -> Fiber {
    let fiber = unsafe {
        kernel32::CreateFiber(
            stack_size as u32,
            Some(fiber_proc),
            Box::into_raw(Box::new(func)) as LPVOID,
        )
    };

    // TODO: Return an error result, rather than just logging a warning.
    if fiber.is_null() {
        println!("ERROR: Failed to create fiber");
    }

    fiber
}

pub fn make_active(fiber: Fiber) {
    unsafe { kernel32::SwitchToFiber(fiber); }
}

/// `data` is secretly a pointer to a `Box<Box<FnBox()>>`.
unsafe extern "system" fn fiber_proc(data: LPVOID) {
    let func = Box::from_raw(data as *mut Box<FnBox()>);
    func();
}
