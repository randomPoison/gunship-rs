extern crate bootstrap_gl as gl;
extern crate gl_util;

use bmp::Bitmap;
use bootstrap::window::Window;
use camera::Camera;
use geometry::mesh::{Mesh, VertexAttribute};
use light::Light;
use material::*;
use math::*;
use self::gl::{
    BufferTarget, ClearBufferMask, Comparison, DestFactor, GlType, IndexType, IntegerName,
    ProgramObject, ServerCapability, ShaderObject, ShaderType, SourceFactor, Texture2dTarget,
    TextureBindTarget, TextureDataType, TextureFormat, TextureInternalFormat, TextureObject,
    UniformLocation};
use self::gl_util::*;
use std::collections::HashMap;
use std::ptr;
use std::mem;
use super::GpuMesh;

#[derive(Debug, Clone)]
pub struct GLRender {
    meshes: HashMap<GpuMesh, MeshData>,
    mesh_id_counter: usize,
}

impl GLRender {
    pub fn new(window: &Window) -> GLRender {
        gl::create_context();

        gl::enable(ServerCapability::DebugOutput);

        let major_version = {
            let mut major_version = 0;
            gl::get_integers(IntegerName::MajorVersion, &mut major_version);
            major_version
        };
        let minor_version = {
            let mut minor_version = 0;
            gl::get_integers(IntegerName::MinorVersion, &mut minor_version);
            minor_version
        };

        if major_version >= 4 && minor_version >= 3 {
            // TODO: Also check if the extension is available for older versions.
            gl::debug_message_callback(gl::debug_callback, ptr::null_mut());
        }

        gl::enable(ServerCapability::DepthTest);
        gl::enable(ServerCapability::CullFace);

        gl::clear_color(0.3, 0.3, 0.3, 1.0);
        gl::viewport(0, 0, 800, 800);

        GLRender {
            meshes: HashMap::new(),
            mesh_id_counter: 0,
        }
    }

    pub fn gen_mesh(&mut self, mesh: &Mesh) -> GpuMesh {
        // Generate array buffer.
        let mut vertex_buffer = VertexBuffer::new();
        vertex_buffer.set_data_f32(mesh.vertex_data());

        let index_buffer = IndexBuffer::new();
        index_buffer.set_data_u32(mesh.indices());

        let mesh_id = GpuMesh(self.mesh_id_counter);
        self.mesh_id_counter += 1;

        self.meshes.insert(mesh_id, MeshData {
            vertex_buffer:      vertex_buffer,
            index_buffer:       index_buffer,
            position_attribute: mesh.position(),
            normal_attribute:   mesh.normal(),
            uv_attribute:       None,
            element_count:      mesh.indices().len(),
        });

        mesh_id
    }

    pub fn gen_texture(&self, bitmap: &Bitmap) -> TextureObject {
        let texture = gl::gen_texture();
        gl::bind_texture(TextureBindTarget::Texture2d, texture);
        gl::texture_image_2d(
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

    pub fn draw_mesh(
        &self,
        mesh: &MeshData,
        material: &Material,
        model_transform: Matrix4,
        normal_transform: Matrix4,
        camera: &Camera,
        lights: &mut Iterator<Item=Light>
    ) {
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

        // Set the shader to use.
        let shader = material.shader();
        shader.set_active();

        // Specify the layout of the vertex data.
        let position_attrib = shader.vertex_position
            .expect("Could not get vertexPosition attribute");
        gl::vertex_attrib_pointer(
            position_attrib,
            3,
            GlType::Float,
            false,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.position_attribute.offset * mem::size_of::<f32>());
        gl::enable_vertex_attrib_array(position_attrib);

        if let Some(mesh_normal) = mesh.normal_attribute {
            if let Some(shader_normal) = shader.vertex_normal {
                gl::vertex_attrib_pointer(
                    shader_normal,
                    3,
                    GlType::Float,
                    false,
                    (mesh_normal.stride * mem::size_of::<f32>()) as i32,
                    mesh_normal.offset * mem::size_of::<f32>());
                gl::enable_vertex_attrib_array(shader_normal);
            }
        }

        // Set uniform transforms.
        if let Some(model_transform_location) = shader.model_transform {
            gl::uniform_matrix_4x4(
                model_transform_location,
                true,
                model_transform.raw_data());
        }

        if let Some(normal_transform_location) = shader.normal_transform {
            gl::uniform_matrix_4x4(
                normal_transform_location,
                true,
                view_normal_transform.raw_data());
        }

        if let Some(view_transform_location) = shader.view_transform {
            gl::uniform_matrix_4x4(
                view_transform_location,
                true,
                view_transform.raw_data());
        }

        if let Some(model_view_transform_location) = shader.model_view_transform {
            gl::uniform_matrix_4x4(
                model_view_transform_location,
                true,
                model_view_transform.raw_data());
        }

        if let Some(projection_transform_location) = shader.projection_transform {
            gl::uniform_matrix_4x4(
                projection_transform_location,
                true,
                projection_transform.raw_data());
        }

        if let Some(model_view_projection_location) = shader.model_view_projection {
            gl::uniform_matrix_4x4(
                model_view_projection_location,
                true,
                model_view_projection.raw_data());
        }

        // Set uniform colors.
        let ambient_color = Color::new(0.25, 0.25, 0.25, 1.0);
        if let Some(ambient_location) = shader.global_ambient {
            gl::uniform_4f(ambient_location, ambient_color.as_array());
        }

        if let Some(camera_position_location) = shader.camera_position {
            gl::uniform_4f(camera_position_location, camera.position.as_array());
        }

        // Apply material attributes.
        for (name, property) in material.properties() {
            match *property {
                MaterialProperty::Color(color) => {

                },
                MaterialProperty::Texture(ref texture) => {

                },
            }
        }

        if let Some(light_position_location) = shader.light_position {
            // Render first light without blending so it overrides any objects behind it.
            // We also render it with light strength 0 so it only renders ambient color.
            {
                gl::uniform_4f(light_position_location, Point::origin().as_array());
                if let Some(light_strength_location) = shader.light_strength {
                    gl::uniform_1f(light_strength_location, 0.0);
                }
                gl::disable(ServerCapability::Blend);
                gl::draw_elements(
                    DrawMode::Triangles,
                    mesh.element_count as i32,
                    IndexType::UnsignedInt,
                    0);
            }

            // Render the rest of the lights with blending on the the depth check set to LEQUAL.
            gl::depth_func(Comparison::LEqual);
            gl::enable(ServerCapability::Blend);
            gl::blend_func(SourceFactor::One, DestFactor::One);

            let ambient_color = Color::new(0.0, 0.0, 0.0, 1.0);
            if let Some(ambient_location) = shader.global_ambient {
                gl::uniform_4f(ambient_location, ambient_color.as_array());
            }

            for light in lights {
                let light_position = match light {
                    Light::Point(ref point_light) => point_light.position
                };
                let light_position_view = light_position * view_transform;

                gl::uniform_4f(light_position_location, light_position_view.as_array());
                if let Some(light_strength_location) = shader.light_strength {
                    // TODO: Use the light's actual strength value.
                    gl::uniform_1f(light_strength_location, 1.0);
                }

                // TODO: Set uniforms and all that jazz.
                DrawBuilder::new(mesh.vertex_buffer, DrawMode::Triangles)
                .index_buffer(mesh.index_buffer)
                .draw();
            }
        } else {
            // Shader doesn't accept lights, so just draw without setting light values.
            // TODO: Set uniforms and all that jazz.
            DrawBuilder::new(mesh.vertex_buffer, DrawMode::Triangles)
            .index_buffer(mesh.index_buffer)
            .draw();
        }

        gl::enable(ServerCapability::DepthTest);
    }

    pub fn draw_wireframe(
        &self,
        camera: &Camera,
        shader: &ShaderProgram,
        mesh: &MeshData,
        model_transform: Matrix4,
        color: Color,
    ) {
        let view_transform = camera.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * model_view_transform;

        shader.set_active();

        // Specify the layout of the vertex data.
        let position_attrib = shader.vertex_position
            .expect("Could not get vertexPosition attribute");
        gl::vertex_attrib_pointer(
            position_attrib,
            3,
            GlType::Float,
            false,
            (mesh.position_attribute.stride * mem::size_of::<f32>()) as i32,
            mesh.position_attribute.offset * mem::size_of::<f32>());
        gl::enable_vertex_attrib_array(position_attrib);

        // Set uniform transforms.
        if let Some(model_transform_location) = shader.model_transform {
            gl::uniform_matrix_4x4(
                model_transform_location,
                true,
                model_transform.raw_data());
        }

        if let Some(view_transform_location) = shader.view_transform {
            gl::uniform_matrix_4x4(
                view_transform_location,
                true,
                view_transform.raw_data());
        }

        if let Some(model_view_transform_location) = shader.model_view_transform {
            gl::uniform_matrix_4x4(
                model_view_transform_location,
                true,
                model_view_transform.raw_data());
        }

        if let Some(projection_transform_location) = shader.projection_transform {
            gl::uniform_matrix_4x4(
                projection_transform_location,
                true,
                projection_transform.raw_data());
        }

        if let Some(model_view_projection_location) = shader.model_view_projection {
            gl::uniform_matrix_4x4(
                model_view_projection_location,
                true,
                model_view_projection.raw_data());
        }

        // if let Some(surface_color_location) = shader.surface_color {
        //     gl::uniform_4f(surface_color_location, color.as_array());
        // }

        DrawBuilder::new(mesh.vertex_buffer, DrawMode::Triangles)
        .index_buffer(mesh.index_buffer)
        .draw();
    }

    /// Clears the current back buffer.
    pub fn clear(&self) {
        gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth);
    }

    /// Swap the front and back buffers for the render system.
    pub fn swap_buffers(&self, window: &Window) {
        gl::swap_buffers(window);
    }

    pub fn compile_shader_program(&self, vert_shader: &str, frag_shader: &str) -> ShaderProgram {
        // TODO: Handle any failure to compile shaders.
        let vs = self.compile_shader(vert_shader, ShaderType::VertexShader);
        let fs = self.compile_shader(frag_shader, ShaderType::FragmentShader);
        let program = self.link_program(vs, fs);
        ShaderProgram::new(program)
    }

    fn compile_shader(&self, shader_source: &str, shader_type: ShaderType) -> ShaderObject {
        let shader = gl::create_shader(shader_type);

        // Attempt to compile the shader
        gl::shader_source(shader, shader_source);
        gl::compile_shader(shader).unwrap(); // TODO: Propogate errors upwards for better handling.

        shader
    }

    fn link_program(&self, vert_shader: ShaderObject, frag_shader: ShaderObject) -> ProgramObject {
        let program = gl::create_program();

        gl::attach_shader(program, vert_shader);
        gl::attach_shader(program, frag_shader);
        gl::link_program(program).unwrap();

        program
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshData {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
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
}

impl ShaderProgram {
    fn new(program_object: ProgramObject) -> ShaderProgram {
        ShaderProgram {
            program_object: program_object,

            vertex_position:       gl::get_attrib_location(program_object, b"vertexPosition\0"),
            vertex_normal:         gl::get_attrib_location(program_object, b"vertexNormal\0"),
            vertex_uv:             gl::get_attrib_location(program_object, b"vertexUv\0"),

            model_transform:       gl::get_uniform_location(program_object, b"modelTransform\0"),
            view_transform:        gl::get_uniform_location(program_object, b"viewTransform\0"),
            model_view_transform:  gl::get_uniform_location(program_object, b"modelViewTransform\0"),
            projection_transform:  gl::get_uniform_location(program_object, b"projectionTransform\0"),
            model_view_projection: gl::get_uniform_location(program_object, b"modelViewProjection\0"),
            view_normal_transform: gl::get_uniform_location(program_object, b"viewNormalTransform\0"),
            normal_transform:      gl::get_uniform_location(program_object, b"normalTransform\0"),
            camera_position:       gl::get_uniform_location(program_object, b"cameraPosition\0"),
            light_position:        gl::get_uniform_location(program_object, b"lightPosition\0"),
            light_strength:        gl::get_uniform_location(program_object, b"lightStrength\0"),
            global_ambient:        gl::get_uniform_location(program_object, b"globalAmbient\0"),
        }
    }

    pub fn set_active(&self) {
        gl::use_program(self.program_object);
    }
}

/// Represents texture data that has been sent to the GPU.
#[derive(Debug, Clone)]
pub struct GpuTexture(TextureObject);

struct ShaderProperty {
    attrib: AttributeLocation,
}
