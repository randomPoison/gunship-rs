use std::ffi::CString;
use std::mem;
use std::ptr;

use gl;
use super::x11::glx;
// use super::x11::xlib;

use window::Window;

pub type GLContext = glx::GLXContext;

pub fn init(_window: &Window) {
    println!("gl::init() is not implemented on linux");
}

pub fn create_context(window: &Window) -> GLContext {
    let context = unsafe {
        let context = glx::glXCreateContext(window.display, window.visual_info, ptr::null_mut(), 1);
        glx::glXMakeCurrent(window.display, window.window, context);
        context
    };

    set_proc_loader();

    context
}

pub fn set_proc_loader() {
    // provide method for loading functions
    gl::load_with(|s| {
        let string = CString::new(s);
        unsafe {
            mem::transmute(glx::glXGetProcAddress(mem::transmute(string.unwrap().as_ptr())))
        }
    });
}

pub fn swap_buffers(window: &Window) {
    unsafe { glx::glXSwapBuffers(window.display, window.window); }
}
