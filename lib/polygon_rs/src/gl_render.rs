use std::ptr;
use std::mem;

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
    shader: ProgramObject,
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

    pub fn draw_mesh(&self, mesh: &GLMeshData, model_transform: Matrix4, normal_transform: Matrix4, camera: &Camera, lights: &mut Iterator<Item=Light>) {
        let gl = &self.gl;
        let view_transform = camera.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * model_view_transform;

        let view_normal_transform = {
            let inverse_model = normal_transform.transpose();
            let inverse_view = camera.inverse_view_matrix();
            let inverse_model_view = inverse_model * inverse_view;
            inverse_model_view.transpose()
        };

        // Bind the buffers for the mesh.
        gl.bind_vertex_array(mesh.vertex_array);
        gl.bind_buffer(BufferTarget::ArrayBuffer, mesh.vertex_buffer);
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, mesh.index_buffer);

        // Set the shader to use.
        gl.use_program(mesh.shader);

        // Specify the layout of the vertex data.
        let possition_attrib = gl.get_attrib(mesh.shader, b"vertexPosition\0")
            .expect("Could not get vertexPosition attribute");
        gl.vertex_attrib_pointer(
            possition_attrib,
            3,
            GLType::Float,
            false,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.position_attribute.offset * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(possition_attrib);

        let normal_attrib = gl.get_attrib(mesh.shader, b"vertexNormal\0")
            .expect("Could not get vertexNormal attribute");
        gl.vertex_attrib_pointer(
            normal_attrib,
            3,
            GLType::Float,
            false,
            (mesh.normal_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.normal_attribute.offset * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(normal_attrib);

        // Set uniform transforms.
        if let Some(model_transform_location) = gl.get_uniform(mesh.shader, b"modelTransform\0") {
            gl.uniform_matrix_4x4(
                model_transform_location,
                true,
                model_transform.raw_data());
        }

        if let Some(normal_transform_location) = gl.get_uniform(mesh.shader, b"normalTransform\0") {
            gl.uniform_matrix_4x4(
                normal_transform_location,
                true,
                view_normal_transform.raw_data());
        }

        if let Some(view_transform_location) = gl.get_uniform(mesh.shader, b"viewTransform\0") {
            gl.uniform_matrix_4x4(
                view_transform_location,
                true,
                view_transform.raw_data());
        }

        if let Some(model_view_transform_location)
            = gl.get_uniform(mesh.shader, b"modelViewTransform\0") {
            gl.uniform_matrix_4x4(
                model_view_transform_location,
                true,
                model_view_transform.raw_data());
        }

        if let Some(projection_transform_location)
            = gl.get_uniform(mesh.shader, b"projectionTransform\0") {
            gl.uniform_matrix_4x4(
                projection_transform_location,
                true,
                projection_transform.raw_data());
        }

        if let Some(model_view_projection_location)
            = gl.get_uniform(mesh.shader, b"modelViewProjection\0") {
            gl.uniform_matrix_4x4(
                model_view_projection_location,
                true,
                model_view_projection.raw_data());
        }

        // Set uniform colors.
        let ambient_color = Color::new(0.25, 0.25, 0.25, 1.0);
        let maybe_ambient_location = gl.get_uniform(mesh.shader, b"globalAmbient\0");
        if let Some(ambient_location) = maybe_ambient_location {
            gl.uniform_4f(ambient_location, ambient_color.as_array());
        }

        if let Some(camera_position_location) = gl.get_uniform(mesh.shader, b"cameraPosition\0") {
            gl.uniform_4f(camera_position_location, camera.position.as_array());
        }

        if let Some(light_position_location) = gl.get_uniform(mesh.shader, b"lightPosition\0") {
            // Render first light without blending so it overrides any objects behind it.
            if let Some(light) = lights.next() {
                let light_position = match light {
                    Light::Point(ref point_light) => point_light.position
                };
                let light_position_view = view_transform * light_position;

                gl.uniform_4f(light_position_location, light_position_view.as_array());

                gl.disable(ServerCapability::Blend);
                gl.draw_elements(
                    DrawMode::Triangles,
                    mesh.element_count as i32,
                    IndexType::UnsignedInt,
                    0);
            }

            // Render the rest of the lights with blending on the the depth check set to LEQUAL.
            gl.depth_func(Comparison::LEqual);
            gl.enable(ServerCapability::Blend);
            gl.blend_func(SourceFactor::One, DestFactor::One);

            let ambient_color = Color::new(0.0, 0.0, 0.0, 1.0);
            if let Some(ambient_location) = maybe_ambient_location {
                gl.uniform_4f(ambient_location, ambient_color.as_array());
            }

            // TODO: What's the deal with this nasty construct? Can we do this with an actual `for` loop?
            loop { match lights.next() {
                Some(light) => {
                    let light_position = match light {
                        Light::Point(ref point_light) => point_light.position
                    };
                    let light_position_view = view_transform * light_position;

                    gl.uniform_4f(light_position_location, light_position_view.as_array());

                    gl.draw_elements(
                        DrawMode::Triangles,
                        mesh.element_count as i32,
                        IndexType::UnsignedInt,
                        0);
                },
                None => break,
            } }
        }

        gl.enable(ServerCapability::DepthTest);

        // Unbind buffers.
        gl.unbind_vertex_array();
        gl.unbind_buffer(BufferTarget::ArrayBuffer);
        gl.unbind_buffer(BufferTarget::ElementArrayBuffer);
    }

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

    fn link_program(&self, vert_shader: ShaderObject, frag_shader: ShaderObject) -> ProgramObject {
        let program = self.gl.create_program();

        self.gl.attach_shader(program, vert_shader);
        self.gl.attach_shader(program, frag_shader);
        self.gl.link_program(program).unwrap();

        program
    }
}
