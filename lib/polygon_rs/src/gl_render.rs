use std::ptr;
use std::mem;
use std::str;
use std::ffi::CString;

use bootstrap::window::Window;
use gl;
use gl::*;

use math::Matrix4;
use math::Color;

use geometry::mesh::{Mesh, VertexAttribute};
use camera::Camera;
use light::Light;

#[derive(Debug, Clone)]
pub struct GLRender {
    gl: gl::Context,
}

#[derive(Debug, Clone, Copy)]
pub struct GLMeshData {
    vertex_array: VertexArrayObject,
    vertex_buffer: VertexBufferObject,
    index_buffer: VertexBufferObject,
    shader: gl::UInt,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: VertexAttribute,
    element_count: usize,
}

// TODO: This should be Drop for GLRender.
// pub fn tear_down(renderer: &GLRender) {
//     gl::destroy_context(renderer.context);
// }

impl GLRender {
    pub fn new(window: &Window) -> GLRender {
        let gl = gl::Context::new(window);

        gl.enable(ServerCapability::DebugOutput);
        gl.debug_message_callback(gl::debug_callback, ptr::null_mut());

        gl.enable(ServerCapability::DepthTest);
        gl.enable(ServerCapability::CullFace);

        gl.clear_color(0.3, 0.3, 0.3, 1.0);

        gl.viewport(0, 0, 800, 800);

        GLRender {
            gl: gl,
        }
    }

    pub fn gen_mesh(&self, mesh: &Mesh, vertex_src: &str, frag_src: &str) -> GLMeshData {
        let gl = &self.gl;

        // generate array buffer
        let mut vertex_array = VertexArrayObject::null();
        gl.gen_vertex_array(&mut vertex_array);
        gl.bind_vertex_array(vertex_array);

        // generate vertex buffer, passing the raw data held by the mesh
        let mut vertex_buffer = VertexBufferObject::null();
        gl.gen_buffer(&mut vertex_buffer);
        gl.bind_buffer(BufferTarget::ArrayBuffer, vertex_buffer);

        gl.buffer_data(
            BufferTarget::ArrayBuffer,
            &*mesh.raw_data,
            BufferUsage::StaticDraw);

        let mut index_buffer = VertexBufferObject::null();
        gl.gen_buffer(&mut index_buffer);
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, index_buffer);

        gl.buffer_data(
            BufferTarget::ElementArrayBuffer,
            &*mesh.indices,
            BufferUsage::StaticDraw);

        // TODO: Handle any failure to compile shaders.
        let vs = self.compile_shader(vertex_src, ShaderType::VertexShader);
        let fs = self.compile_shader(frag_src, ShaderType::FragmentShader);
        let program = self.link_program(vs, fs);

        // Unbind buffers.
        gl.bind_vertex_array(VertexArrayObject::null());
        gl.bind_buffer(BufferTarget::ArrayBuffer, VertexBufferObject::null());
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, VertexBufferObject::null());

        GLMeshData {
            vertex_array: vertex_array,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            shader: program,
            position_attribute: mesh.position_attribute,
            normal_attribute: mesh.normal_attribute,
            element_count: mesh.indices.len(),
        }
    }

    pub fn draw_mesh(&self, mesh: &GLMeshData, model_transform: Matrix4, normal_transform: Matrix4, camera: &Camera, lights: &mut Iterator<Item=Light>) { unsafe {
        // let view_transform = camera.view_matrix();
        // let model_view_transform = view_transform * model_transform;
        // let projection_transform = camera.projection_matrix();
        // let model_view_projection = projection_transform * ( view_transform * model_transform );
        //
        // let view_normal_transform = {
        //     let inverse_model = normal_transform.transpose();
        //     let inverse_view = camera.inverse_view_matrix();
        //     let inverse_model_view = inverse_model * inverse_view;
        //     inverse_model_view.transpose()
        // };
        //
        // // Bind the buffers for the mesh.
        // gl.bind_vertex_array(mesh.vertex_array);
        // gl.bind_buffer(BufferTarget::ArrayBuffer, mesh.vertex_buffer);
        // gl.bind_buffer(BufferTarget::ElementArrayBuffer, mesh.index_buffer);
        //
        // // Set the shader to use.
        // gl::UseProgram(mesh.shader);
        //
        // // Specify the layout of the vertex data.
        // let position_location = gl::GetAttribLocation(
        //     mesh.shader,
        //     CString::new("vertexPosition").unwrap().as_ptr()); // TODO: Write a helper to make using cstrings easier.
        // gl::VertexAttribPointer(
        //     position_location as gl::UInt,
        //     3,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
        //     mem::transmute(mesh.position_attribute.offset * mem::size_of::<f32>()));
        // gl::EnableVertexAttribArray(position_location as gl::UInt);
        //
        // let normal_location = gl::GetAttribLocation(
        //     mesh.shader,
        //     CString::new("vertexNormal").unwrap().as_ptr());
        // gl::VertexAttribPointer(
        //     normal_location as gl::UInt,
        //     3,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     (mesh.normal_attribute.stride * mem::size_of::<f32>()) as i32,
        //     mem::transmute(mesh.normal_attribute.offset * mem::size_of::<f32>()));
        // gl::EnableVertexAttribArray(normal_location as gl::UInt);
        //
        // // Set uniform transforms.
        // let model_transform_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("modelTransform").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     model_transform_location,
        //     1,
        //     gl::TRUE,
        //     model_transform.raw_data());
        //
        // let normal_transform_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("normalTransform").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     normal_transform_location,
        //     1,
        //     gl::TRUE,
        //     view_normal_transform.raw_data());
        //
        // let view_transform_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("viewTransform").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     view_transform_location,
        //     1,
        //     gl::TRUE,
        //     view_transform.raw_data());
        //
        // let model_view_transform_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("modelViewTransform").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     model_view_transform_location,
        //     1,
        //     gl::TRUE,
        //     model_view_transform.raw_data());
        //
        // let projection_transform_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("projectionTransform").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     projection_transform_location,
        //     1,
        //     gl::TRUE,
        //     projection_transform.raw_data());
        //
        // let model_view_projection_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("modelViewProjection").unwrap().as_ptr());
        // gl::UniformMatrix4fv(
        //     model_view_projection_location,
        //     1,
        //     gl::TRUE,
        //     model_view_projection.raw_data());
        //
        // // Set uniform colors.
        // let ambient_color = Color::new(0.25, 0.25, 0.25, 1.0);
        // let ambient_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("globalAmbient").unwrap().as_ptr());
        // gl::Uniform4fv(ambient_location, 1, ambient_color.raw_data());
        //
        // let camera_position_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("cameraPosition").unwrap().as_ptr());
        // gl::Uniform4fv(camera_position_location, 1, camera.position.raw_data());
        //
        // let light_position_location =
        //     gl::GetUniformLocation(mesh.shader, CString::new("lightPosition").unwrap().as_ptr());
        //
        // // Render first light without blending so it overrides any objects behind it.
        // if let Some(light) = lights.next() {
        //     let light_position = match light {
        //         Light::Point(ref point_light) => point_light.position
        //     };
        //     let light_position_view = view_transform * light_position;
        //
        //     gl::Uniform4fv(light_position_location, 1, light_position_view.raw_data());
        //
        //     gl::Disable(gl::BLEND);
        //     gl::DrawElements(gl::TRIANGLES,
        //                      mesh.element_count as GLsizei,
        //                      gl::UNSIGNED_INT,
        //                      0 as *const GLvoid);
        // }
        //
        // // Render the rest of the lights with blending on the the depth check set to LEQUAL.
        // gl::DepthFunc(gl::LEQUAL);
        // gl::Enable(gl::BLEND);
        // gl::BlendFunc(gl::ONE, gl::ONE);
        //
        // let ambient_color = Color::new(0.0, 0.0, 0.0, 1.0);
        // gl::Uniform4fv(ambient_location, 1, ambient_color.raw_data());
        //
        // loop { match lights.next() {
        //     Some(light) => {
        //         let light_position = match light {
        //             Light::Point(ref point_light) => point_light.position
        //         };
        //         let light_position_view = view_transform * light_position;
        //
        //         gl::Uniform4fv(light_position_location, 1, light_position_view.raw_data());
        //
        //         gl::DrawElements(gl::TRIANGLES,
        //                          mesh.element_count as GLsizei,
        //                          gl::UNSIGNED_INT,
        //                          0 as *const GLvoid);
        //     },
        //     None => break,
        // } }
        //
        // gl::Enable(gl::DEPTH_TEST);
        //
        // // Unbind buffers.
        // gl.bind_vertex_array(0);
        // gl.bind_buffer(BufferTarget::ArrayBuffer, 0);
        // gl.bind_buffer(BufferTarget::ElementArrayBuffer, 0);
    } }

    /// Clears the current back buffer.
    pub fn clear(&self) {
        self.gl.clear(ClearBufferMask::Color | ClearBufferMask::Depth);
    }

    /// Swap the front and back buffers for the render system.
    pub fn swap_buffers(&self, window: &Window) {
        self.gl.swap_buffers(window);
    }

    fn compile_shader(&self, shader_source: &str, shader_type: ShaderType) -> ShaderObject {
        let shader = self.gl.create_shader(shader_type);

        // Attempt to compile the shader
        self.gl.shader_source(shader, shader_source);
        self.gl.compile_shader(shader).unwrap(); // TODO: Propogate errors upwards for better handling.

        shader
    }

    fn link_program(&self, vs: ShaderObject, fs: ShaderObject) -> gl::UInt {
        // unsafe {
        //     let program = gl::CreateProgram();
        //     gl::AttachShader(program, vs);
        //     gl::AttachShader(program, fs);
        //     gl::LinkProgram(program);
        //
        //     // Get the link status
        //     let mut status = gl::FALSE as GLint;
        //     gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        //
        //     // Fail on error
        //     if status != (gl::TRUE as GLint) {
        //         let mut len: GLint = 0;
        //         gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        //         let mut buf = Vec::with_capacity(len as usize);
        //         buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        //         gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        //         panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
        //     }
        //
        //     program
        // }

        0
    }
}
