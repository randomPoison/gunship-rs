extern crate kernel32;
extern crate winapi;

use ::{Fiber, PREV};
use std::mem;
use std::ptr;
use std::raw::TraitObject;
use self::winapi::*;

pub type PlatformId = LPVOID;

pub fn init() -> PlatformId {
    let fiber = unsafe { kernel32::ConvertThreadToFiber(ptr::null_mut()) };

    if fiber.is_null() {
        println!("ERROR: Failed to convert main thread to a fiber");
    }

    fiber
}

pub fn create_fiber<F>(stack_size: usize, func: F) -> PlatformId
    where
    F: Fn(Fiber),
    F: 'static + Send,
{
    let boxed_func = Box::new(func) as Box<Fn(Fiber)>;

    // `box_fun` is two pointers, so it can't be passed directly. We need to box it again
    // so that we have a single pointer to the trait object that we can use when the fiber
    // is started.
    let func_ptr = Box::into_raw(Box::new(boxed_func)) as LPVOID;

    let fiber = unsafe {
        kernel32::CreateFiber(
            stack_size as u32,
            Some(fiber_proc),
            func_ptr,
        )
    };

    // TODO: Return an error result, rather than just logging a warning.
    if fiber.is_null() {
        panic!("ERROR: Failed to create fiber");
    }

    fiber
}

/// Makes `fiber` active, then returns the handle of the fiber that resumed the current one.
pub unsafe fn resume(fiber: PlatformId) {
    kernel32::SwitchToFiber(fiber);
}

/// `data` is secretly a pointer to a `Box<Box<FnBox()>>`.
unsafe extern "system" fn fiber_proc(data: LPVOID) {
    let func = *Box::from_raw(data as *mut Box<Fn(Fiber)>);
    let prev_fiber = PREV.with(|prev| prev.get().expect("PREV was None in fiber_proc()"));

    func(Fiber(prev_fiber));
}
