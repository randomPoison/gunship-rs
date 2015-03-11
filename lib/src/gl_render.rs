use std::ptr;
use std::str;
use std::ffi::CString;

use gl;
use gl::types::*;

// reexport platform-specific versions of functions
#[cfg(target_family = "windows")]
pub use windows::gl_render::{init_opengl, create_gl_context, swap_buffers};

pub static VERTEX_SHADER_SRC: &'static str = r#"
#version 150

in vec4 vertexPos;

void main(void)
{
    gl_Position = vertexPos;
}"#;

pub static FRAGMENT_SHADER_SRC: &'static str = r#"
#version 150

out vec4 fragmentColor;

void main(void)
{
    fragmentColor = vec4(1, 0, 0, 1);
}"#;

pub struct Mesh;

impl Mesh {
    pub fn new() -> Mesh {
        Mesh
    }
}

pub fn draw_mesh(mesh: &Mesh) {
    unsafe {
        // TODO rebind the buffers or whatever

        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw a triangle from the 3 vertices
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 5);

        swap_buffers(); // TODO don't swap buffers after every draw
    }
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
