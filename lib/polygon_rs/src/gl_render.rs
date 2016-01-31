use bmp::Bitmap;
use bootstrap::window::Window;
use camera::Camera;
use geometry::mesh::{Mesh, VertexAttribute};
use gl;
use gl::*;
use light::Light;
use math::*;
use std::ptr;
use std::mem;

#[derive(Debug, Clone)]
pub struct GLRender {
    gl: gl::Context,
}

impl GLRender {
    pub fn new(window: &Window) -> GLRender {
        let gl = gl::Context::new(window);

        // TODO: Once better logging is implemented leave logging in and just disable unwanted logs.
        // let version_str = gl.get_string(StringName::Version);
        // println!("OpenGL Version: {:?}", version_str);

        gl.enable(ServerCapability::DebugOutput);

        let major_version = gl.get_integer(IntegerName::MajorVersion);
        let minor_version = gl.get_integer(IntegerName::MinorVersion);

        if major_version >= 4 && minor_version >= 3 {
            // TODO: Also check if the extension is available for older versions.
            gl.debug_message_callback(gl::debug_callback, ptr::null_mut());
        }

        gl.enable(ServerCapability::DepthTest);
        gl.enable(ServerCapability::CullFace);

        gl.clear_color(0.3, 0.3, 0.3, 1.0);

        gl.viewport(0, 0, 800, 800);

        GLRender {
            gl: gl,
        }
    }

    pub fn gen_mesh(&self, mesh: &Mesh) -> GLMeshData {
        let gl = &self.gl;

        // Generate array buffer.
        let vertex_array = gl.gen_vertex_array();
        gl.bind_vertex_array(vertex_array);

        // Generate vertex buffer, passing the raw data held by the mesh.
        let vertex_buffer = gl.gen_buffer();
        gl.bind_buffer(BufferTarget::ArrayBuffer, vertex_buffer);

        gl.buffer_data(
            BufferTarget::ArrayBuffer,
            mesh.vertex_data(),
            BufferUsage::StaticDraw);

        let index_buffer = gl.gen_buffer();
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, index_buffer);

        gl.buffer_data(
            BufferTarget::ElementArrayBuffer,
            mesh.indices(),
            BufferUsage::StaticDraw);

        // Unbind buffers.
        gl.bind_vertex_array(VertexArrayObject::null());
        gl.bind_buffer(BufferTarget::ArrayBuffer, VertexBufferObject::null());
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, VertexBufferObject::null());

        GLMeshData {
            vertex_array:       vertex_array,
            vertex_buffer:      vertex_buffer,
            index_buffer:       index_buffer,
            position_attribute: mesh.position(),
            normal_attribute:   mesh.normal(),
            uv_attribute:       None,
            element_count:      mesh.indices().len(),
        }
    }

    pub fn gen_texture(&self, bitmap: &Bitmap) -> TextureObject {
        let gl = &self.gl;

        let texture = gl.gen_texture();
        gl.bind_texture(TextureBindTarget::Texture2d, texture);
        gl.texture_image_2d(
            Texture2dTarget::Texture2d,
            0,
            TextureInternalFormat::Rgba,
            bitmap.width() as i32,
            bitmap.height() as i32,
            0,
            TextureFormat::Bgra,
            TextureDataType::UnsignedByte,
            bitmap.data().as_ptr() as *const _);

        texture
    }

    /// SUPER BAD LACK OF SAFETY, should be using RAII and some proper resource management, but
    /// that will have to wait until we get a real rendering system.
    pub fn delete_mesh(&self, mesh: GLMeshData) {
        self.gl.delete_buffer(mesh.vertex_buffer);
        self.gl.delete_buffer(mesh.index_buffer);
        self.gl.delete_vertex_array(mesh.vertex_array);
    }

    pub fn draw_mesh(
        &self,
        mesh: &GLMeshData,
        shader: &ShaderProgram,
        model_transform: Matrix4,
        normal_transform: Matrix4,
        camera: &Camera,
        lights: &mut Iterator<Item=Light>
    ) {
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
        shader.set_active(gl);

        // Specify the layout of the vertex data.
        let position_attrib = shader.vertex_position
            .expect("Could not get vertexPosition attribute");
        gl.vertex_attrib_pointer(
            position_attrib,
            3,
            GLType::Float,
            false,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.position_attribute.offset * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(position_attrib);

        let normal_attrib = shader.vertex_normal
            .expect("Could not get vertexNormal attribute");
        gl.vertex_attrib_pointer(
            normal_attrib,
            3,
            GLType::Float,
            false,
            (mesh.normal_attribute.unwrap().stride * mem::size_of::<f32>()) as i32,
            mesh.normal_attribute.unwrap().offset * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(normal_attrib);

        // Set uniform transforms.
        if let Some(model_transform_location) = shader.model_transform {
            gl.uniform_matrix_4x4(
                model_transform_location,
                true,
                model_transform.raw_data());
        }

        if let Some(normal_transform_location) = shader.normal_transform {
            gl.uniform_matrix_4x4(
                normal_transform_location,
                true,
                view_normal_transform.raw_data());
        }

        if let Some(view_transform_location) = shader.view_transform {
            gl.uniform_matrix_4x4(
                view_transform_location,
                true,
                view_transform.raw_data());
        }

        if let Some(model_view_transform_location) = shader.model_view_transform {
            gl.uniform_matrix_4x4(
                model_view_transform_location,
                true,
                model_view_transform.raw_data());
        }

        if let Some(projection_transform_location) = shader.projection_transform {
            gl.uniform_matrix_4x4(
                projection_transform_location,
                true,
                projection_transform.raw_data());
        }

        if let Some(model_view_projection_location) = shader.model_view_projection {
            gl.uniform_matrix_4x4(
                model_view_projection_location,
                true,
                model_view_projection.raw_data());
        }

        // Set uniform colors.
        let ambient_color = Color::new(0.25, 0.25, 0.25, 1.0);
        if let Some(ambient_location) = shader.global_ambient {
            gl.uniform_4f(ambient_location, ambient_color.as_array());
        }

        if let Some(camera_position_location) = shader.camera_position {
            gl.uniform_4f(camera_position_location, camera.position.as_array());
        }

        if let Some(light_position_location) = shader.light_position {
            // Render first light without blending so it overrides any objects behind it.
            // We also render it with light strength 0 so it only renders ambient color.
            {
                gl.uniform_4f(light_position_location, Point::origin().as_array());
                if let Some(light_strength_location) = shader.light_strength {
                    gl.uniform_1f(light_strength_location, 0.0);
                }
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
            if let Some(ambient_location) = shader.global_ambient {
                gl.uniform_4f(ambient_location, ambient_color.as_array());
            }

            for light in lights {
                let light_position = match light {
                    Light::Point(ref point_light) => point_light.position
                };
                let light_position_view = light_position * view_transform;

                gl.uniform_4f(light_position_location, light_position_view.as_array());
                if let Some(light_strength_location) = shader.light_strength {
                    // TODO: Use the light's actual strength value.
                    gl.uniform_1f(light_strength_location, 1.0);
                }

                gl.draw_elements(
                    DrawMode::Triangles,
                    mesh.element_count as i32,
                    IndexType::UnsignedInt,
                    0);
            }
        }

        gl.enable(ServerCapability::DepthTest);

        // Unbind buffers.
        gl.unbind_vertex_array();
        gl.unbind_buffer(BufferTarget::ArrayBuffer);
        gl.unbind_buffer(BufferTarget::ElementArrayBuffer);
    }

    pub fn draw_line(&self, camera: &Camera, shader: &ShaderProgram, start: Point, end: Point) {
        let gl = &self.gl;
        let buffer = gl.gen_buffer();
        gl.bind_buffer(BufferTarget::ArrayBuffer, buffer);

        gl.buffer_data(
            BufferTarget::ArrayBuffer,
            &[start.x, start.y, start.z, end.x, end.y, end.z],
            BufferUsage::StaticDraw);

        shader.set_active(gl);

        let position_attrib = shader.vertex_position
            .expect("Could not get vertexPosition attribute");
        gl.vertex_attrib_pointer(
            position_attrib,
            3,
            GLType::Float,
            false,
            (3 * mem::size_of::<f32>()) as i32,
            0 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(position_attrib);

        let view_transform = camera.view_matrix();
        let projection_transform = camera.projection_matrix();
        let view_projection = projection_transform * view_transform;
        if let Some(model_view_projection_location) = shader.model_view_projection {
            gl.uniform_matrix_4x4(
                model_view_projection_location,
                true,
                view_projection.raw_data());
        }

        if let Some(surface_color_location) = shader.surface_color {
            gl.uniform_4f(surface_color_location, Color::new(1.0, 1.0, 1.0, 1.0).as_array());
        }

        gl.draw_arrays(DrawMode::Lines, 0, 6);

        gl.unbind_buffer(BufferTarget::ArrayBuffer);
    }

    pub fn draw_wireframe(
        &self,
        camera: &Camera,
        shader: &ShaderProgram,
        mesh: &GLMeshData,
        model_transform: Matrix4,
        color: Color,
    ) {
        let gl = &self.gl;
        let view_transform = camera.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * model_view_transform;

        shader.set_active(gl);

        // Bind the buffers for the mesh.
        gl.bind_vertex_array(mesh.vertex_array);
        gl.bind_buffer(BufferTarget::ArrayBuffer, mesh.vertex_buffer);
        gl.bind_buffer(BufferTarget::ElementArrayBuffer, mesh.index_buffer);

        // Specify the layout of the vertex data.
        let position_attrib = shader.vertex_position
            .expect("Could not get vertexPosition attribute");
        gl.vertex_attrib_pointer(
            position_attrib,
            3,
            GLType::Float,
            false,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.position_attribute.offset * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(position_attrib);

        // Set uniform transforms.
        if let Some(model_transform_location) = shader.model_transform {
            gl.uniform_matrix_4x4(
                model_transform_location,
                true,
                model_transform.raw_data());
        }

        if let Some(view_transform_location) = shader.view_transform {
            gl.uniform_matrix_4x4(
                view_transform_location,
                true,
                view_transform.raw_data());
        }

        if let Some(model_view_transform_location) = shader.model_view_transform {
            gl.uniform_matrix_4x4(
                model_view_transform_location,
                true,
                model_view_transform.raw_data());
        }

        if let Some(projection_transform_location) = shader.projection_transform {
            gl.uniform_matrix_4x4(
                projection_transform_location,
                true,
                projection_transform.raw_data());
        }

        if let Some(model_view_projection_location) = shader.model_view_projection {
            gl.uniform_matrix_4x4(
                model_view_projection_location,
                true,
                model_view_projection.raw_data());
        }

        if let Some(surface_color_location) = shader.surface_color {
            gl.uniform_4f(surface_color_location, color.as_array());
        }

        gl.draw_elements(
            DrawMode::Lines,
            mesh.element_count as i32,
            IndexType::UnsignedInt,
            0);

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

    pub fn compile_shader_program(&self, vert_shader: &str, frag_shader: &str) -> ShaderProgram {
        // TODO: Handle any failure to compile shaders.
        let vs = self.compile_shader(vert_shader, ShaderType::VertexShader);
        let fs = self.compile_shader(frag_shader, ShaderType::FragmentShader);
        let program = self.link_program(vs, fs);
        ShaderProgram::new(program, &self.gl)
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

#[derive(Debug, Clone, Copy)]
pub struct GLMeshData {
    vertex_array: VertexArrayObject,
    vertex_buffer: VertexBufferObject,
    index_buffer: VertexBufferObject,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: Option<VertexAttribute>,
    pub uv_attribute: Option<VertexAttribute>,
    element_count: usize,
}

#[derive(Debug, Clone)]
pub struct ShaderProgram {
    program_object:        ProgramObject,

    vertex_position:       Option<AttributeLocation>,
    vertex_normal:         Option<AttributeLocation>,
    vertex_uv:             Option<AttributeLocation>,

    model_transform:       Option<UniformLocation>,
    view_transform:        Option<UniformLocation>,
    model_view_transform:  Option<UniformLocation>,
    projection_transform:  Option<UniformLocation>,
    model_view_projection: Option<UniformLocation>,
    view_normal_transform: Option<UniformLocation>,
    normal_transform:      Option<UniformLocation>,

    light_position:        Option<UniformLocation>,
    light_strength:        Option<UniformLocation>,
    global_ambient:        Option<UniformLocation>,

    camera_position:       Option<UniformLocation>,
    surface_color:         Option<UniformLocation>,
}

impl ShaderProgram {
    fn new(program_object: ProgramObject, gl: &gl::Context) -> ShaderProgram {
        ShaderProgram {
            program_object: program_object,

            vertex_position:       gl.get_attrib(program_object, b"vertexPosition\0"),
            vertex_normal:         gl.get_attrib(program_object, b"vertexNormal\0"),
            vertex_uv:             gl.get_attrib(program_object, b"vertexUv\0"),

            model_transform:       gl.get_uniform(program_object, b"modelTransform\0"),
            view_transform:        gl.get_uniform(program_object, b"viewTransform\0"),
            model_view_transform:  gl.get_uniform(program_object, b"modelViewTransform\0"),
            projection_transform:  gl.get_uniform(program_object, b"projectionTransform\0"),
            model_view_projection: gl.get_uniform(program_object, b"modelViewProjection\0"),
            view_normal_transform: gl.get_uniform(program_object, b"viewNormalTransform\0"),
            normal_transform:      gl.get_uniform(program_object, b"normalTransform\0"),
            camera_position:       gl.get_uniform(program_object, b"cameraPosition\0"),
            light_position:        gl.get_uniform(program_object, b"lightPosition\0"),
            light_strength:        gl.get_uniform(program_object, b"lightStrength\0"),
            surface_color:         gl.get_uniform(program_object, b"surfaceColor\0"),
            global_ambient:        gl.get_uniform(program_object, b"globalAmbient\0"),
        }
    }

    pub fn set_active(&self, gl: &gl::Context) {
        gl.use_program(self.program_object);
    }
}
