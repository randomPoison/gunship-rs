use std::ptr;
use std::mem;
use std::str;
use std::ffi::CString;

use gl;
use gl::types::*;

use bootstrap::window::Window;
use bootstrap::gl_utils::{self, GLContext};

use math::point::Point;
use math::matrix::Matrix4;

use geometry::mesh::Mesh;
use geometry::face::Face;
use camera::Camera;

#[allow(dead_code)] #[derive(Clone, Copy)]
pub struct GLRender {
    context: GLContext // TODO: do we need to hold onto the context?
}

#[derive(Debug)]
pub struct GLMeshData {
    array_buffer: GLuint,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    shader: GLuint,
    element_count: usize
}

// TODO: This should be GLRender::new() for consistency.
pub fn init(window: &Window) -> GLRender {
    gl_utils::init(window);
    let context = gl_utils::create_context(window);

    // do some basic configuration stuff
    unsafe {
        // Enable depth testing
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); // TODO: Do we need to clear here?

        // Enable backface culling
        gl::Enable(gl::CULL_FACE);

        gl::Viewport(0, 0, 800, 800);
    }

    GLRender {
        context: context
    }
}

// TODO: This should be Drop for GLRender.
// pub fn tear_down(renderer: &GLRender) {
//     gl_utils::destroy_context(renderer.context);
// }

impl GLRender {
    pub fn gen_mesh(&self, mesh: &Mesh, vertex_src: &str, frag_src: &str) -> GLMeshData {

        // generate array buffer
        let mut array_buffer = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut array_buffer);
            gl::BindVertexArray(array_buffer);
        }

        // generate vertex buffer,
        // passing the raw data held by the mesh
        let mut vertex_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

            gl::BufferData(gl::ARRAY_BUFFER,
                           (mesh.vertices.len() * mem::size_of::<Point>()) as GLsizeiptr,
                           mem::transmute(&(mesh.vertices[0].x)),
                           gl::STATIC_DRAW);
        }

        let mut index_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (mesh.faces.len() * mem::size_of::<Face>()) as GLsizeiptr,
                           mem::transmute(&(mesh.faces[0].indices[0])),
                           gl::STATIC_DRAW);
        }

        // TODO: do some handling of errors here?
        let vs = GLRender::compile_shader(vertex_src, gl::VERTEX_SHADER);
        let fs = GLRender::compile_shader(frag_src, gl::FRAGMENT_SHADER);
        let program = GLRender::link_program(vs, fs);

        unsafe {
            // Unbind buffers.
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        GLMeshData {
            array_buffer: array_buffer,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            shader: program,
            element_count: mesh.faces.len() * 3
        }
    }

    /// TODO: make this a member of GLMeshData?
    pub fn draw_mesh(&self, mesh: &GLMeshData, model_transform: Matrix4, camera: &Camera) { unsafe {
        let view_transform = camera.view_transform();
        let projection = camera.projection_matrix();

        // Bind the buffers for the mesh.
        gl::BindVertexArray(mesh.array_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vertex_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.index_buffer);

        // Set the shader to use.
        gl::UseProgram(mesh.shader);

        // Specify the layout of the vertex data.
        let vertex_pos_location = gl::GetAttribLocation(
            mesh.shader,
            CString::new("vertexPosition").unwrap().as_ptr()); // TODO: Write a helper to make using cstrings easier.
        gl::VertexAttribPointer(
            vertex_pos_location as GLuint,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Point>() as GLsizei,
            ptr::null());
        gl::EnableVertexAttribArray(vertex_pos_location as GLuint);

        let model_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("modelTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(model_transform_location,
                             1,
                             gl::TRUE,
                             model_transform.raw_data());

        let view_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("viewTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(view_transform_location,
                             1,
                             gl::TRUE,
                             view_transform.raw_data());

        let projection_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("projectionTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(projection_transform_location,
                             1,
                             gl::TRUE,
                             projection.raw_data());

        let model_view_projection = projection * ( view_transform * model_transform );
        let model_view_projection_location =
            gl::GetUniformLocation(mesh.shader, CString::new("modelViewProjection").unwrap().as_ptr());
        gl::UniformMatrix4fv(model_view_projection_location,
                             1,
                             gl::TRUE,
                             model_view_projection.raw_data());

        gl::DrawElements(gl::TRIANGLES,
                         mesh.element_count as GLsizei,
                         gl::UNSIGNED_INT,
                         0 as *const GLvoid);

        // Unbind buffers.
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    } }

    /// Clears the current back buffer.
    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// Swap the front and back buffers for the render system.
    pub fn swap_buffers(&self) {
        gl_utils::swap_buffers();
    }

    fn compile_shader(src: &str, ty: GLenum) -> GLuint {
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

    fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
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
}
