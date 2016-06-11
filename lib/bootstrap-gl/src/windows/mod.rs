extern crate winapi;
extern crate opengl32;
extern crate gdi32;
extern crate user32;

use std::{mem, ptr};
use std::ffi::CString;

use self::winapi::*;

pub type Context = HGLRC;

pub unsafe fn init() {
    let device_context = {
        let hwnd = user32::GetActiveWindow();
        user32::GetDC(hwnd)
    };

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

    let pixelformat = gdi32::ChoosePixelFormat(device_context, &pfd);

    gdi32::SetPixelFormat(device_context, pixelformat, &pfd);

    // Create and destroy temporary OpenGL context. This is necessary because of a quirk in the
    // way OpenGL works on windows.
    destroy_context(create_context());
}

pub unsafe fn create_context() -> Context {
    let device_context = {
        let hwnd = user32::GetActiveWindow();
        user32::GetDC(hwnd)
    };

    // create context and make it current
    let context = {
        let render_context = opengl32::wglCreateContext(device_context);
        opengl32::wglMakeCurrent(device_context, render_context);
        render_context
    };

    context
}

pub unsafe fn destroy_context(context: Context) {
    let device_context = {
        let hwnd = user32::GetActiveWindow();
        user32::GetDC(hwnd)
    };
    let render_context = opengl32::wglCreateContext(device_context);

    opengl32::wglMakeCurrent(ptr::null_mut(), render_context);
    opengl32::wglDeleteContext(context);
}

pub unsafe fn load_proc(proc_name: &str) -> Option<extern "C" fn()> {
    let string = CString::new(proc_name);
    let cstr = string.unwrap().as_ptr();
    let ptr = opengl32::wglGetProcAddress(cstr);
    Some(mem::transmute(ptr))
}

pub unsafe fn swap_buffers() {
    gdi32::SwapBuffers(opengl32::wglGetCurrentDC()); // TODO maybe pass in the DC?
}
