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

pub fn create_fiber(stack_size: usize, func: &Fn(Fiber)) -> PlatformId {
    let fiber = unsafe {
        let trait_obj: TraitObject = mem::transmute(func);

        kernel32::CreateFiber(
            stack_size as u32,
            Some(fiber_proc),
            Box::into_raw(Box::new(trait_obj)) as LPVOID,
        )
    };

    // TODO: Return an error result, rather than just logging a warning.
    if fiber.is_null() {
        println!("ERROR: Failed to create fiber");
    }

    fiber
}

/// Makes `fiber` active, then returns the handle of the fiber that resumed the current one.
pub fn resume(fiber: PlatformId) {
    unsafe { kernel32::SwitchToFiber(fiber); }
}

/// `data` is secretly a pointer to a `Box<Box<FnBox()>>`.
unsafe extern "system" fn fiber_proc(data: LPVOID) {
    let trait_obj = *Box::from_raw(data as *mut TraitObject);
    let func: &Fn(Fiber) = mem::transmute(trait_obj);

    let prev_fiber = PREV.with(|prev| prev.get().expect("PREV was None in fiber_proc()"));
    func(Fiber(prev_fiber));
}
