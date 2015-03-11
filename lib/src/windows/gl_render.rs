use std::ptr;
use std::mem;
use std::ffi::CString;

use winapi::{HGLRC, PIXELFORMATDESCRIPTOR, WORD,
             PFD_DRAW_TO_WINDOW, PFD_TYPE_RGBA, PFD_MAIN_PLANE, PFD_DOUBLEBUFFER, PFD_SUPPORT_OPENGL};
use opengl32;
use gdi32;

use gl;
use gl::types::*;

use windows::window::Window;
use gl_render::{compile_shader, link_program,
                VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC};

pub fn init_opengl(window: &Window) {
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

    let success = unsafe {
        let pixelformat = gdi32::ChoosePixelFormat(device_context, &pfd);
        gdi32::SetPixelFormat(device_context, pixelformat, &pfd);
        let render_context = opengl32::wglCreateContext(device_context);
        opengl32::wglMakeCurrent(device_context, render_context)
    };

    println!("{:?}", success);

    gl::load_with(|s| {
        let string = CString::new(s);
        unsafe {
            opengl32::wglGetProcAddress(string.unwrap().as_ptr())
        }
    });

    println!("{:?}", gl::DrawArrays::is_loaded());
}

#[allow(unused_variables)]
pub fn create_gl_context(window: &Window) -> HGLRC {
    let device_context = window.dc;

    // TODO create a proper gl context

    let mut array_buffer = 0;
    let mut vertex_buffer = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut array_buffer);
        gl::BindVertexArray(array_buffer);
    }

    let vertex_data: [GLfloat; 15] =
    [ 0.00, 0.00, 0.00,
      0.50, 0.00, 0.00,
      0.50, 0.50, 0.00,
      0.25, 0.75, 0.00,
      0.00, 0.50, 0.00 ];

    unsafe {
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertex_data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&vertex_data[0]),
                       gl::STATIC_DRAW);
    }

        let vs = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);

    unsafe {
        gl::UseProgram(program);
        // gl::BindFragDataLocation(program, 0,
        //                          CString::new(b"fragmentColor").unwrap().as_ptr());

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(program,
                                             CString::new(b"vertexPos").unwrap().as_ptr());
        gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT,
                                gl::FALSE as GLboolean, 0, ptr::null());
        gl::EnableVertexAttribArray(pos_attr as GLuint);
    }

    ptr::null_mut()
}

pub fn swap_buffers()
{
    unsafe {
        gdi32::SwapBuffers(opengl32::wglGetCurrentDC()); // TODO maybe pass in the DC?
    }
}
