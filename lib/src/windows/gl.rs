use std::ptr;
use std::mem;
use std::ffi::CString;

use windows::winapi::{
    HGLRC, PIXELFORMATDESCRIPTOR, WORD,
    PFD_DRAW_TO_WINDOW, PFD_TYPE_RGBA, PFD_MAIN_PLANE, PFD_DOUBLEBUFFER, PFD_SUPPORT_OPENGL
};
use windows::opengl32;
use windows::gdi32;

use gl;

use windows::window::Window;

pub fn init(window: &Window) {
    let device_context = window.dc;

    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
        nVersion: 1,
        dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
        iPixelType: PFD_TYPE_RGBA,
        cColorBits: 16,
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
        cDepthBits: 16,
        cStencilBits: 1,
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
        opengl32::wglMakeCurrent(device_context, render_context)
    };

    gl::load_with(|s| {
        let string = CString::new(s);
        unsafe {
            opengl32::wglGetProcAddress(string.unwrap().as_ptr())
        }
    });
}

#[allow(unused_variables)]
pub fn create_context(window: &Window) -> HGLRC {
    let device_context = window.dc;

    // TODO create a proper gl context
    ptr::null_mut()
}

pub fn swap_buffers()
{
    unsafe {
        gdi32::SwapBuffers(opengl32::wglGetCurrentDC()); // TODO maybe pass in the DC?
    }
}
