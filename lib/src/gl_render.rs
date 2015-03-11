use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

use gl;
use gl::types::*;
use winapi::{HGLRC, PIXELFORMATDESCRIPTOR, WORD,
             PFD_DRAW_TO_WINDOW, PFD_TYPE_RGBA, PFD_MAIN_PLANE, PFD_DOUBLEBUFFER, PFD_SUPPORT_OPENGL};
use opengl32;
use gdi32;

use window::Window;

static VERTEX_SHADER_SRC: &'static str = "\
#version 150                            \n\
                                        \n\
in vec4 vertexPos;                      \n\
                                        \n\
void main(void)                         \n\
{                                       \n\
gl_Position = vertexPos;                \n\
}";

static FRAGMENT_SHADER_SRC: &'static str = "\
#version 150                              \n\
                                          \n\
out vec4 fragmentColor;                   \n\
                                          \n\
void main(void)                           \n\
{                                         \n\
fragmentColor = vec4(1, 0, 0, 1);         \n\
}";

pub struct Mesh;

impl Mesh {
    pub fn new() -> Mesh {
        Mesh
    }
}

#[allow(unused_variables)]
pub fn draw_mesh(mesh: &Mesh) {
    unsafe {
        // TODO rebind the buffers or whatever

        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw a triangle from the 3 vertices
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 5);

        gdi32::SwapBuffers(opengl32::wglGetCurrentDC());
    }
}

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

        let vertex_data: [GLfloat; 15] =
        [ 0.00, 0.00, 0.00,
          0.50, 0.00, 0.00,
          0.50, 0.50, 0.00,
          0.25, 0.75, 0.00,
          0.00, 0.50, 0.00 ];

        gl::GenBuffers(1, &mut vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertex_data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&vertex_data[0]),
                       gl::STATIC_DRAW);

        let vs = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);

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

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);

        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes());
        gl::ShaderSource(shader, 1, &c_str.unwrap().as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ShaderInfoLog not valid utf8"));
        }

        shader
    }
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
        }
        program
    }
}
