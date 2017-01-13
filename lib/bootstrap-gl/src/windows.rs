extern crate winapi;
extern crate opengl32;
extern crate gdi32;
extern crate user32;
extern crate kernel32;

use std::{mem, ptr};

use self::winapi::*;

pub type DeviceContext = HDC;
pub type Context = (HDC, HGLRC);

pub unsafe fn create_context(device_context: DeviceContext) -> Option<Context> {
    let tmp_context = opengl32::wglCreateContext(device_context);
    if tmp_context.is_null() {
        return None;
    }

    make_current((device_context, tmp_context));

    let render_context = create_context_attribs(device_context, ptr::null_mut(), ptr::null());

    clear_current();
    opengl32::wglDeleteContext(tmp_context);

    if render_context.is_null() {
        let error = kernel32::GetLastError();
        println!("WARNING: Failed to created OpenGL context, last error: {:#x}", error);
        None
    } else {
        make_current((device_context, render_context));

        // TODO: Don't do this in context creation.
        if set_swap_interval(0) != ::types::Boolean::True {
            println!("WARNING: Failed to set swap interval of setting swap interval");
        }

        clear_current();

        Some((device_context, render_context))
    }
}

pub unsafe fn destroy_context(context: Context) {
    let (_, render_context) = context;
    clear_current();

    let result = opengl32::wglDeleteContext(render_context);

    assert!(result == 1, "Failed to delete context: {:?}", render_context);
}

pub unsafe fn load_proc(proc_name: &str) -> Option<extern "system" fn()> {
    let string = proc_name.as_bytes();
    debug_assert!(
        string[string.len() - 1] == 0,
        "Proc name \"{}\" is not null terminated",
        proc_name,
    );

    let mut ptr = opengl32::wglGetProcAddress(string.as_ptr() as *const _);

    if ptr.is_null() {
        let module = kernel32::LoadLibraryA(b"opengl32.dll\0".as_ptr() as *const _);

        // TODO: What do we want to do in this case? Probably just return `None`, right?
        assert!(!module.is_null(), "Failed to load opengl32.dll");

        ptr = kernel32::GetProcAddress(module, string.as_ptr() as *const _);
    }

    if ptr.is_null() {
        let actual_dc = opengl32::wglGetCurrentDC();
        let actual_context = opengl32::wglGetCurrentContext();
        println!(
            "pointer for {} was null, last error: 0x{:X}, active dc: {:?}, active context: {:?}",
            proc_name,
            kernel32::GetLastError(),
            actual_dc,
            actual_context,
        );

        return None;
    }

    Some(mem::transmute(ptr))
}

pub unsafe fn swap_buffers(context: Context) {
    let (device_context, _) = context;
    if gdi32::SwapBuffers(device_context) != TRUE {
        let (device_context, render_context) = context;
        let hwnd = user32::GetActiveWindow();
        panic!(
            "Swap buffers failed, dc: {:?}, context: {:?} last error: 0x:{:X}, hwnd: {:?}",
            device_context,
            render_context,
            kernel32::GetLastError(),
            hwnd,
        );
    }
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

gl_proc!(wglGetExtensionsStringARB:
    fn get_extension_string(hdc: ::platform::winapi::HDC) -> *const u8);

gl_proc!(wglCreateContextAttribsARB:
    fn create_context_attribs(
        hdc: ::platform::winapi::HDC,
        share_context: ::platform::winapi::HGLRC,
        attrib_list: *const i32
    ) -> ::platform::winapi::HGLRC);

gl_proc!(wglGetSwapIntervalEXT:
    fn get_swap_interval() -> i32);

gl_proc!(wglSwapIntervalEXT:
    fn set_swap_interval(interval: i32) -> ::types::Boolean);
