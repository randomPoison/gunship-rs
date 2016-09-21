extern crate winapi;
extern crate opengl32;
extern crate gdi32;
extern crate user32;
extern crate kernel32;

use std::{mem, ptr};
use std::ffi::CString;

use self::winapi::*;

pub type DeviceContext = HDC;
pub type Context = (HDC, HGLRC);

pub unsafe fn create_context(device_context: DeviceContext) -> Option<Context> {
    let render_context = opengl32::wglCreateContext(device_context);
    if render_context.is_null() {
        let error = kernel32::GetLastError();
        println!("WARNING: Failed to created OpenGL context, last error: {:#x}", error);
        None
    } else {
        Some((device_context, render_context))
    }
}

pub unsafe fn destroy_context(context: Context) {
    let (_, render_context) = context;
    let result = opengl32::wglMakeCurrent(::std::ptr::null_mut(), ::std::ptr::null_mut());
    assert!(result == 1, "Failed to clear the current context");

    let result = opengl32::wglDeleteContext(render_context);

    assert!(result == 1, "Failed to delete context: {:?}", render_context);
}

pub unsafe fn load_proc(proc_name: &str) -> Option<extern "system" fn()> {
    let string = CString::new(proc_name).unwrap();
    let cstr = string.as_ptr();
    let ptr = opengl32::wglGetProcAddress(cstr);

    if ptr.is_null() {
        let actual_dc = opengl32::wglGetCurrentDC();
        let actual_context = opengl32::wglGetCurrentContext();
        println!(
            "pointer for {} was null, last error: 0x{:X}, active dc: {:?}, active context: {:?}",
            proc_name,
            kernel32::GetLastError(),
            actual_dc,
            actual_context);
    }

    Some(mem::transmute(ptr))
}

pub unsafe fn swap_buffers(context: Context) {
    let (device_context, _) = context;
    gdi32::SwapBuffers(device_context);
}

pub unsafe fn make_current(context: Context) -> Context {
    let old_device_context = opengl32::wglGetCurrentDC();
    let old_render_context = opengl32::wglGetCurrentContext();

    let (device_context, render_context) = context;
    let result = opengl32::wglMakeCurrent(device_context, render_context);
    if result != TRUE {
        let hwnd = user32::GetActiveWindow();
        panic!(
            "Failed to make context current, dc: {:?}, context: {:?} last error: 0x:{:X}, actual dc and context: {:?} and {:?}, hwnd: {:?}",
            device_context,
            render_context,
            kernel32::GetLastError(),
            old_device_context,
            old_render_context,
            hwnd,
        );
    }

    (old_device_context, old_render_context)
}

pub unsafe fn clear_current() {
    make_current((ptr::null_mut(), ptr::null_mut()));
}
