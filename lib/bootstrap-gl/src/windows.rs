extern crate winapi;
extern crate opengl32;
extern crate gdi32;

use std::mem;
use std::ffi::CString;

use self::winapi::*;

use bootstrap::window::Window;

pub type Context = HGLRC;

pub fn init(window: &Window) {
    let device_context = window.dc;

    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
        nVersion: 1,
        dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
        iPixelType: PFD_TYPE_RGBA,
        cColorBits: 32,
        cRedBits: 0,
        cRedShift: 0,
        cGreenBits: 0,
        cGreenShift: 0,
        cBlueBits: 0,
        cBlueShift: 0,
        cAlphaBits: 0,
        cAlphaShift: 0,
        cAccumBits: 0,
        cAccumRedBits: 0,
        cAccumGreenBits: 0,
        cAccumBlueBits: 0,
        cAccumAlphaBits: 0,
        cDepthBits: 24,
        cStencilBits: 8,
        cAuxBuffers: 0,
        iLayerType: PFD_MAIN_PLANE,
        bReserved: 0,
        dwLayerMask: 0,
        dwVisibleMask: 0,
        dwDamageMask: 0
    };

    unsafe {
        let pixelformat = gdi32::ChoosePixelFormat(device_context, &pfd);
        gdi32::SetPixelFormat(device_context, pixelformat, &pfd);
        let render_context = opengl32::wglCreateContext(device_context);
        opengl32::wglDeleteContext(render_context);
    };
}

pub fn create_context(window: &Window) -> Context {
    let device_context = window.dc;

    // create context and make it current
    let context = unsafe {
        let render_context = opengl32::wglCreateContext(device_context);
        opengl32::wglMakeCurrent(device_context, render_context);
        render_context
    };

    context
}

pub fn destroy_context(context: Context) {
    unsafe {
        opengl32::wglDeleteContext(context);
    }
}

pub fn proc_loader(proc_name: &str) -> Option<extern "C" fn()> {
    let string = CString::new(proc_name);
    Some(unsafe {
        mem::transmute(opengl32::wglGetProcAddress(string.unwrap().as_ptr()))
    })
}

pub fn swap_buffers(_window: &Window) {
    unsafe {
        gdi32::SwapBuffers(opengl32::wglGetCurrentDC()); // TODO maybe pass in the DC?
    }
}
