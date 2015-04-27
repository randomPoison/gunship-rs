use std::ptr;
use std::mem;
use std::str;
use std::ffi::CString;

use gl;
use gl::types::*;

use bootstrap::window::Window;
use bootstrap::gl_utils::{self, GLContext};

use math::Point;
use math::Matrix4;
use math::Color;

use geometry::mesh::{Mesh, VertexAttribute};
use geometry::face::Face;
use camera::Camera;

#[allow(dead_code)] #[derive(Clone, Copy)]
pub struct GLRender {
    context: GLContext // TODO: do we need to hold onto the context?
}

#[derive(Debug, Clone, Copy)]
pub struct GLMeshData {
    array_buffer: GLuint,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    shader: GLuint,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: VertexAttribute,
    element_count: usize,
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

        // generate vertex buffer, passing the raw data held by the mesh
        let mut vertex_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

            gl::BufferData(gl::ARRAY_BUFFER,
                           (mesh.raw_data.len() * mem::size_of::<f32>()) as GLsizeiptr,
                           mem::transmute(&mesh.raw_data[0]),
                           gl::STATIC_DRAW);
        }

        let mut index_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (mesh.indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                           mem::transmute(&(mesh.indices[0])),
                           gl::STATIC_DRAW);
        }

        // TODO: do some handling of errors here?
        let vs = GLRender::compile_shader(vertex_src, gl::VERTEX_SHADER);
        let fs = GLRender::compile_shader(frag_src, gl::FRAGMENT_SHADER);
        let program = GLRender::link_program(vs, fs);

        // Unbind buffers.
        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        GLMeshData {
            array_buffer: array_buffer,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            shader: program,
            position_attribute: mesh.position_attribute,
            normal_attribute: mesh.normal_attribute,
            element_count: mesh.indices.len(),
        }
    }

    pub fn draw_mesh(&self, mesh: &GLMeshData, model_transform: Matrix4, normal_transform: Matrix4, camera: &Camera) { unsafe {
        let view_transform = camera.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * ( view_transform * model_transform );

        let view_normal_transform = {
            let inverse_model = normal_transform.transpose();
            let inverse_view = camera.inverse_view_matrix();
            let inverse_model_view = inverse_model * inverse_view;
            inverse_model_view.transpose()
        };

        let light_position = view_transform * Point::origin();

        // Bind the buffers for the mesh.
        gl::BindVertexArray(mesh.array_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vertex_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.index_buffer);

        // Set the shader to use.
        gl::UseProgram(mesh.shader);

        // Specify the layout of the vertex data.
        let position_location = gl::GetAttribLocation(
            mesh.shader,
            CString::new("vertexPosition").unwrap().as_ptr()); // TODO: Write a helper to make using cstrings easier.
        gl::VertexAttribPointer(
            position_location as GLuint,
            3,
            gl::FLOAT,
            gl::FALSE,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mem::transmute(mesh.position_attribute.offset * mem::size_of::<f32>()));
        gl::EnableVertexAttribArray(position_location as GLuint);

        let normal_location = gl::GetAttribLocation(
            mesh.shader,
            CString::new("vertexNormal").unwrap().as_ptr());
        gl::VertexAttribPointer(
            normal_location as GLuint,
            3,
            gl::FLOAT,
            gl::FALSE,
            (mesh.normal_attribute.stride * mem::size_of::<f32>()) as i32,
            mem::transmute(mesh.normal_attribute.offset * mem::size_of::<f32>()));
        gl::EnableVertexAttribArray(normal_location as GLuint);

        // Set uniform transforms.
        let model_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("modelTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            model_transform_location,
            1,
            gl::TRUE,
            model_transform.raw_data());

        let normal_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("normalTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            normal_transform_location,
            1,
            gl::TRUE,
            view_normal_transform.raw_data());

        let view_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("viewTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            view_transform_location,
            1,
            gl::TRUE,
            view_transform.raw_data());

        let model_view_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("modelViewTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            model_view_transform_location,
            1,
            gl::TRUE,
            model_view_transform.raw_data());

        let projection_transform_location =
            gl::GetUniformLocation(mesh.shader, CString::new("projectionTransform").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            projection_transform_location,
            1,
            gl::TRUE,
            projection_transform.raw_data());

        let model_view_projection_location =
            gl::GetUniformLocation(mesh.shader, CString::new("modelViewProjection").unwrap().as_ptr());
        gl::UniformMatrix4fv(
            model_view_projection_location,
            1,
            gl::TRUE,
            model_view_projection.raw_data());

        // Set uniform colors.
        let ambient_color = Color::new(0.5, 0.5, 0.5, 1.0);
        let ambient_location =
            gl::GetUniformLocation(mesh.shader, CString::new("globalAmbient").unwrap().as_ptr());
        gl::Uniform4fv(ambient_location, 1, ambient_color.raw_data());

        // Light stuffs.
        let light_position_location =
            gl::GetUniformLocation(mesh.shader, CString::new("lightPosition").unwrap().as_ptr());
        gl::Uniform4fv(light_position_location, 1, light_position.raw_data());

        let camera_position_location =
            gl::GetUniformLocation(mesh.shader, CString::new("cameraPosition").unwrap().as_ptr());
        gl::Uniform4fv(camera_position_location, 1, camera.position.raw_data());

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
