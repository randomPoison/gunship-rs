pub extern crate gl_util;

use {BuildMaterialError, Counter, GpuMesh, Renderer};
use anchor::*;
use bootstrap::window::Window;
use camera::*;
use geometry::mesh::{Mesh, VertexAttribute};
use light::*;
use material::*;
use mesh_instance::*;
use math::*;
use self::gl_util::*;
use self::gl_util::context::{Context, Error as ContextError};
use self::gl_util::shader::*;
use self::gl_util::shader::Shader as GlShader;
use self::gl_util::texture::{
    Texture2d as GlTexture2d,
    TextureFormat,
    TextureInternalFormat,
};
use shader::Shader;
use std::collections::HashMap;
use std::str;
use stopwatch::Stopwatch;
use texture::*;

static DEFAULT_SHADER_BYTES: &'static [u8] = include_bytes!("../../resources/materials/diffuse_lit.material");

#[derive(Debug)]
pub struct GlRender {
    context: Context,

    shared_materials: HashMap<MaterialId, Material>,
    meshes: HashMap<GpuMesh, MeshData>,
    textures: HashMap<GpuTexture, GlTexture2d>,
    mesh_instances: HashMap<MeshInstanceId, MeshInstance>,
    anchors: HashMap<AnchorId, Anchor>,
    cameras: HashMap<CameraId, Camera>,
    lights: HashMap<LightId, Light>,
    programs: HashMap<Shader, Program>,

    mesh_instances_with_shared_materials: HashMap<MaterialId, Vec<MeshInstanceId>>,
    mesh_instances_with_owned_material: Vec<MeshInstanceId>,

    material_counter: MaterialId,
    mesh_counter: GpuMesh,
    texture_counter: GpuTexture,
    mesh_instance_counter: MeshInstanceId,
    anchor_counter: AnchorId,
    camera_counter: CameraId,
    light_counter: LightId,
    shader_counter: Shader,

    ambient_color: Color,

    default_material: Material,
}

impl GlRender {
    pub fn new(window: &Window) -> Result<GlRender, Error> {
        let _s = Stopwatch::new("Initializing OpenGl renderer");

        let context = {
            let _s = Stopwatch::new("Creating context");
            Context::from_window(window)?
        };

        {
            let _s = Stopwatch::new("Clearing buffer");
            context.clear();
        }

        let mut renderer = GlRender {
            context: context,

            shared_materials: HashMap::new(),
            meshes: HashMap::new(),
            textures: HashMap::new(),
            mesh_instances: HashMap::new(),
            anchors: HashMap::new(),
            cameras: HashMap::new(),
            lights: HashMap::new(),
            programs: HashMap::new(),

            mesh_instances_with_shared_materials: HashMap::new(),
            mesh_instances_with_owned_material: Vec::new(),

            material_counter: MaterialId::initial(),
            mesh_counter: GpuMesh::initial(),
            texture_counter: GpuTexture::initial(),
            mesh_instance_counter: MeshInstanceId::initial(),
            anchor_counter: AnchorId::initial(),
            camera_counter: CameraId::initial(),
            light_counter: LightId::initial(),
            shader_counter: Shader::initial(),

            ambient_color: Color::rgb(0.01, 0.01, 0.01),

            // Use temporary value and replace it later.
            default_material: Material::new(Shader::initial()),
        };

        // Load source code for the default material.
        let default_material_source = str::from_utf8(DEFAULT_SHADER_BYTES).unwrap();
        let material_source = MaterialSource::from_str(default_material_source).unwrap();

        // default_material.set_color("surface_color", Color::new(0.25, 0.25, 0.25, 1.0));
        // default_material.set_color("surface_specular", Color::new(1.0, 1.0, 1.0, 1.0));
        // default_material.set_f32("surface_shininess", 3.0);

        // Create the default material and drop add it to the renderer.
        let default_material = renderer.build_material(material_source).unwrap();
        renderer.default_material = default_material;

        Ok(renderer)
    }

    fn render_mesh_instance(
        &self,
        mesh_instance: &MeshInstance,
        material: &Material,
        camera: &Camera,
        camera_anchor: &Anchor,
        has_setup_lights: &mut bool,
        has_setup_material: &mut bool,
    ) {
        let _s = Stopwatch::new("Rendering mesh instance");

        let anchor = match mesh_instance.anchor() {
            Some(anchor_id) => self.anchors.get(&anchor_id).expect("No such anchor exists"),
            None => return,
        };

        let model_transform = anchor.matrix();
        let normal_transform = anchor.normal_matrix();

        let mesh_data = self.meshes.get(mesh_instance.mesh()).expect("Mesh data does not exist for mesh id");

        let default_texture = GlTexture2d::empty(&self.context);

        // Calculate the various transforms needed for rendering.
        let view_transform = camera_anchor.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * model_view_transform;

        let view_normal_transform = {
            let inverse_model = normal_transform.transpose();
            let inverse_view = camera_anchor.inverse_view_matrix().into();
            let inverse_model_view = inverse_model * inverse_view;
            inverse_model_view.transpose()
        };

        // The data for the light uniforms must be declared before `draw_builder` so they
        // can outlive it, since they are borrowed when the uniforms are set.
        let mut light_type = [0i32; 8];
        let mut light_strength = [0.0f32; 8];
        let mut light_color = [Color::rgb(0.0, 0.0, 0.0); 8];
        let mut light_position = [Point::origin(); 8];
        let mut light_position_view = [Point::origin(); 8];
        let mut light_radius = [0.0f32; 8];
        let mut light_direction = [Vector3::zero(); 8];
        let mut light_direction_view = [Vector3::zero(); 8];

        let program = self
            .programs
            .get(material.shader())
            .expect("Material is using a shader that does not exist");

        // Set the shader to use.
        let mut draw_builder = DrawBuilder::new(
            &self.context,
            &mesh_data.vertex_array,
            DrawMode::Triangles,
        );

        draw_builder
        .program(program)
        .cull(Face::Back)
        .depth_test(Comparison::Less);

        // Set uniform transforms.
        {
            let _stopwatch = Stopwatch::new("Transform uniforms");

            draw_builder
            .uniform(
                "model_transform",
                GlMatrix {
                    data: model_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "normal_transform",
                GlMatrix {
                    data: normal_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "view_normal_transform",
                GlMatrix {
                    data: view_normal_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "view_transform",
                GlMatrix {
                    data: view_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "model_view_transform",
                GlMatrix {
                    data: model_view_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "projection_transform",
                GlMatrix {
                    data: projection_transform.raw_data(),
                    transpose: true,
                },
            )
            .uniform(
                "model_view_projection",
                GlMatrix {
                    data: model_view_projection.raw_data(),
                    transpose: true,
                },
            );
        }

        // Apply material attributes.
        if !*has_setup_material {
            let _stopwatch = Stopwatch::new("Material uniforms");

            *has_setup_material = true;

            // Set uniform colors.
            draw_builder.uniform::<[f32; 4]>("global_ambient", self.ambient_color.into());

            // Other uniforms.
            draw_builder.uniform("camera_position", *camera_anchor.position().as_array());

            for (name, property) in material.properties() {
                match *property {
                    MaterialProperty::Color(ref color) => {
                        draw_builder.uniform::<[f32; 4]>(name, color.into());
                    },
                    MaterialProperty::f32(value) => {
                        draw_builder.uniform(name, value);
                    },
                    MaterialProperty::Vector3(value) => {
                        draw_builder.uniform::<[f32; 3]>(name, value.into());
                    },
                    MaterialProperty::Texture(ref texture) => {
                        let gl_texture =
                        self.textures
                        .get(texture)
                        .unwrap_or(&default_texture);
                        draw_builder.uniform(name, gl_texture);
                    },
                }
            }
        }

        // Render all lights in a single pass by sending 8 lights at once in arrays.
        // All light-related uniforms will stay the same for all draws for a given camera,
        // so we only specify them for the first draw and leave them the same after that.
        if !*has_setup_lights {
            *has_setup_lights = true;

            let _stopwatch = Stopwatch::new("Setup lights");

            // TODO: Support having more than 8 lights active at a time. Maybe pick the 8
            // most relevant lights? Or simply support more lights at once in the shader.
            for (index, light) in self.lights.values().take(8).enumerate() {
                // Setup common light data.
                light_color[index] = light.color;
                light_strength[index] = light.strength;

                // Setup data specific to the current type of light.
                match light.data {
                    LightData::Point { radius } => {
                        // Get the light's anchor.
                        let light_anchor = match light.anchor() {
                            Some(anchor_id) => self.anchors.get(&anchor_id).expect("No such anchor exists"),
                            None => panic!("Cannot render point light if it's not attached to an anchor"),
                        };

                        light_type[index] = 1;
                        light_position[index] = light_anchor.position();
                        light_position_view[index] = light_anchor.position() * view_transform;
                        light_radius[index] = radius;
                    },

                    LightData::Directional { direction } => {
                        light_type[index] = 2;
                        light_direction[index] = direction;
                        light_direction_view[index] = direction * view_transform;
                    },
                }
            }

            draw_builder.uniform("light_type", &light_type[..]);
            draw_builder.uniform("light_strength", &light_strength[..]);
            draw_builder.uniform("light_color", Color::as_slice_of_arrays(&light_color));
            draw_builder.uniform("light_position", Point::as_slice_of_arrays(&light_position));
            draw_builder.uniform("light_position_view", Point::as_slice_of_arrays(&light_position_view));
            draw_builder.uniform("light_radius", &light_radius[..]);
            draw_builder.uniform("light_direction", Vector3::as_slice_of_arrays(&light_direction));
            draw_builder.uniform("light_direction_view", Vector3::as_slice_of_arrays(&light_direction_view));
        }

        {
            let _s = Stopwatch::new("Draw mesh");

            draw_builder.draw();
        }
    }
}

impl Drop for GlRender {
    fn drop(&mut self) {
        // Empty all containers to force cleanup of OpenGL primitives before we tear down the
        // GL subsystem.
        // TODO: Do we have to do this? It would be better if we could tear down the context
        // without having to cleanup each GL resource, since deleting the context effectively
        // deletes them all too. I think the problem here comes from the fact that by default
        // the context gets dropped first, then the resources get dropped, and they can't be
        // deleted once the context is gone. If we could get them to silently do nothing when
        // dropped if the context has already been dropped, then we'd get faster shutdown.
        self.shared_materials.clear();
        self.meshes.clear();
        self.textures.clear();
        self.mesh_instances.clear();
        self.anchors.clear();
        self.cameras.clear();
        self.lights.clear();
        self.programs.clear();
    }
}

impl Renderer for GlRender {
    fn draw(&mut self) {
        let _stopwatch = Stopwatch::new("GLRender::draw()");

        {
            let _stopwatch = Stopwatch::new("Clearing buffer");
            self.context.clear();
        }

        // TODO: Support rendering multiple cameras.
        // TODO: Should we warn if there are no cameras?
        if let Some(camera) = self.cameras.values().next() {
            let _stopwatch = Stopwatch::new("Rendering camera");

            let camera_anchor = match camera.anchor() {
                Some(ref anchor_id) => self.anchors.get(anchor_id).expect("No such anchor exists"),
                None => unimplemented!(),
            };

            let mut has_setup_lights = false;

            // Render shared materials first.
            for (material_id, mesh_instances) in &self.mesh_instances_with_shared_materials {
                let _s = Stopwatch::new("Rendering shared material");

                let material = self.shared_materials.get(material_id).expect("No such material exists");
                let mut has_setup_material = false;

                for mesh_instance_id in mesh_instances {
                    let mesh_instance = self.mesh_instances.get(mesh_instance_id).expect("No such mesh instance");
                    self.render_mesh_instance(
                        mesh_instance,
                        material,
                        camera,
                        camera_anchor,
                        &mut has_setup_lights,
                        &mut has_setup_material,
                    );
                }
            }

            // Render meshes with unique materials.
            for mesh_instance_id in &self.mesh_instances_with_owned_material {
                let mesh_instance = self.mesh_instances.get(mesh_instance_id).expect("No such mesh instance");
                let material = mesh_instance.material().expect("Mesh instance was in wrong bucket (was in the owned material bucket, had shared material)");
                self.render_mesh_instance(
                    mesh_instance,
                    material,
                    camera,
                    camera_anchor,
                    &mut false,
                    &mut false,
                );
            }
        }

        {
            let _stopwatch = Stopwatch::new("Swap buffers");
            self.context.swap_buffers();
        }
    }

    fn default_material(&self) -> Material {
        self.default_material.clone()
    }

    fn build_material(&mut self, source: MaterialSource) -> Result<Material, BuildMaterialError> {
        use polygon_material::material_source::PropertyType;

        // COMPILE SHADER SOURCE
        // =====================

        // Generate uniform declarations for the material's properties. This string will be
        // injected into the shader templates.
        let uniform_declarations = {
            let mut uniform_declarations = String::new();
            for property in &source.properties {
                uniform_declarations.push_str("uniform ");

                let type_str = match property.property_type {
                    PropertyType::Color => "vec4",
                    PropertyType::Texture2d => "sampler2D",
                    PropertyType::f32 => "float",
                    PropertyType::Vector3 => "vec3",
                };

                uniform_declarations.push_str(type_str);
                uniform_declarations.push(' ');
                uniform_declarations.push_str(&*property.name);
                uniform_declarations.push_str(";\n");
            }

            uniform_declarations
        };

        static BUILT_IN_UNIFORMS: &'static str = r#"
            uniform mat4 model_transform;
            uniform mat3 normal_transform;
            uniform mat4 view_transform;
            uniform mat3 view_normal_transform;
            uniform mat4 model_view_transform;
            uniform mat4 projection_transform;
            uniform mat4 model_view_projection;

            uniform vec4 global_ambient;
            uniform vec4 camera_position;

            uniform int light_type[8];
            uniform vec4 light_position[8];
            uniform vec4 light_position_view[8];
            uniform float light_strength[8];
            uniform vec4 light_color[8];
            uniform float light_radius[8];
            uniform vec3 light_direction[8];
            uniform vec3 light_direction_view[8];
        "#;

        // Generate the GLSL source for the vertex shader.
        let vert_shader = {
            static DEFAULT_VERT_MAIN: &'static str = r#"
                @position = model_view_projection * vertex_position;

                @vertex.position = vertex_position;
                @vertex.normal = vertex_normal;
                @vertex.uv0 = vertex_uv0;

                @vertex.world_position = model_transform * vertex_position;
                @vertex.world_normal = normalize(normal_transform * vertex_normal);

                @vertex.view_position = model_view_transform * vertex_position;
                @vertex.view_normal = normalize(view_normal_transform * vertex_normal);
            "#;

            // Retrieve source string for the vertex shader.
            let raw_source =
                source
                .programs
                .iter()
                .find(|program_source| program_source.is_vertex())
                .map(|program_source| program_source.source())
                .unwrap_or(DEFAULT_VERT_MAIN);

            // Perform text replacements for the various keywords.
            let replaced_source = raw_source
                .replace("@position", "gl_Position")
                .replace("@vertex.position", "_vertex_position_")
                .replace("@vertex.normal", "_vertex_normal_")
                .replace("@vertex.uv0", "_vertex_uv0_")
                .replace("@vertex.world_position", "_vertex_world_position_")
                .replace("@vertex.world_normal", "_vertex_world_normal_")
                .replace("@vertex.view_position", "_vertex_view_position_")
                .replace("@vertex.view_normal", "_vertex_view_normal_");
            let replaced_source = format!(r#"
                    #version 330 core

                    {}

                    {}

                    layout(location = 0) in vec4 vertex_position;
                    layout(location = 1) in vec3 vertex_normal;
                    layout(location = 2) in vec2 vertex_uv0;

                    out vec4 _vertex_position_;
                    out vec3 _vertex_normal_;
                    out vec2 _vertex_uv0_;
                    out vec4 _vertex_world_position_;
                    out vec3 _vertex_world_normal_;
                    out vec4 _vertex_view_position_;
                    out vec3 _vertex_view_normal_;

                    void main(void) {{
                        {}
                    }}
                "#,
                BUILT_IN_UNIFORMS,
                uniform_declarations,
                replaced_source);

            GlShader::new(&self.context, replaced_source, ShaderType::Vertex).map_err(|err| BuildMaterialError)?
        };

        // Generate the GLSL source for the fragment shader.
        let frag_shader = {
            // Retrieve source string for the fragment shader.
            let raw_source =
                source
                .programs
                .iter()
                .find(|program_source| program_source.is_fragment())
                .map(|program_source| program_source.source())
                .ok_or(BuildMaterialError)?;

            // Perform text replacements for the various keywords.
            let replaced_source = raw_source
                .replace("@color", "_fragment_color_")
                .replace("@vertex.position", "_vertex_position_")
                .replace("@vertex.normal", "_vertex_normal_")
                .replace("@vertex.uv0", "_vertex_uv0_")
                .replace("@vertex.world_position", "_vertex_world_position_")
                .replace("@vertex.world_normal", "_vertex_world_normal_")
                .replace("@vertex.view_position", "_vertex_view_position_")
                .replace("@vertex.view_normal", "_vertex_view_normal_");
            let replaced_source = format!(r#"
                    #version 330 core

                    {}

                    {}

                    in vec4 _vertex_position_;
                    in vec3 _vertex_normal_;
                    in vec2 _vertex_uv0_;
                    in vec4 _vertex_world_position_;
                    in vec3 _vertex_world_normal_;
                    in vec4 _vertex_view_position_;
                    in vec3 _vertex_view_normal_;

                    out vec4 _fragment_color_;

                    void main(void) {{
                        {}
                    }}
                "#,
                BUILT_IN_UNIFORMS,
                uniform_declarations,
                replaced_source);

            GlShader::new(&self.context, replaced_source, ShaderType::Fragment).map_err(|err| BuildMaterialError)?
        };

        let program = Program::new(&self.context, &[vert_shader, frag_shader]).map_err(|err| BuildMaterialError)?;

        let program_id = self.shader_counter.next();
        self.programs.insert(program_id, program);

        // BUILD MATERIAL OBJECT
        // =====================

        let mut material = Material::new(program_id);

        // Add the properties from the material declaration.
        for property in source.properties {
            match property.property_type {
                PropertyType::Color => material.set_color(property.name, Color::default()),
                PropertyType::Texture2d => material.set_texture(property.name, GpuTexture::default()),
                PropertyType::f32 => material.set_f32(property.name, f32::default()),
                PropertyType::Vector3 => material.set_vector3(property.name, Vector3::default()),
            };
        }

        Ok(material)
    }

    fn register_shared_material(&mut self, material: Material) -> MaterialId {
        let material_id = self.material_counter.next();

        let old = self.shared_materials.insert(material_id, material);
        assert!(old.is_none());

        // Ensure there's a bucket for the shared material's mesh instances.
        self.mesh_instances_with_shared_materials.entry(material_id).or_insert(Vec::new());

        material_id
    }

    fn get_material(&self, material_id: MaterialId) -> Option<&Material> {
        self.shared_materials.get(&material_id)
    }

    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh {
        // Configure vertex attributes.
        let position = mesh.position();

        let mesh_id = self.mesh_counter.next();

        let mut vertex_array = VertexArray::with_index_buffer(
            &self.context,
            mesh.vertex_data(),
            mesh.indices(),
        );
        vertex_array.set_attrib(AttributeLocation::from_index(0), position.into());

        if let Some(normal) = mesh.normal() {
            vertex_array.set_attrib(AttributeLocation::from_index(1), normal.into());
        }

        // TODO: Support multiple texcoords.
        if let Some(texcoord) = mesh.texcoord().first().cloned() {
            vertex_array.set_attrib(AttributeLocation::from_index(2), texcoord.into());
        }

        self.meshes.insert(
            mesh_id,
            MeshData {
                vertex_array: vertex_array,
                position_attribute: mesh.position(),
                normal_attribute: mesh.normal(),
                uv_attribute: None,
                element_count: mesh.indices().len(),
            });

        mesh_id
    }

    fn register_texture(&mut self, texture: &Texture2d) -> GpuTexture {
        let (format, internal_format) = match texture.format() {
            DataFormat::Rgb => (TextureFormat::Rgb, TextureInternalFormat::Rgb),
            DataFormat::Rgba => (TextureFormat::Rgba, TextureInternalFormat::Rgba),
            DataFormat::Bgr => (TextureFormat::Bgr, TextureInternalFormat::Rgb),
            DataFormat::Bgra => (TextureFormat::Bgra, TextureInternalFormat::Rgba),
        };

        // Create the Texture2d from the texture data.
        let texture_result = match texture.data() {
            &TextureData::f32(ref data) => {
                GlTexture2d::new(
                    &self.context,
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8(ref data) => {
                GlTexture2d::new(
                    &self.context,
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8x3(ref data) => {
                GlTexture2d::new(
                    &self.context,
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8x4(ref data) => {
                GlTexture2d::new(
                    &self.context,
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
        };
        let gl_texture = texture_result.expect("Unable to send texture to GPU");

        // Register the mesh internally.
        let texture_id = self.texture_counter.next();

        let old = self.textures.insert(texture_id, gl_texture);
        assert!(old.is_none());

        texture_id
    }

    fn register_mesh_instance(&mut self, mesh_instance: MeshInstance) -> MeshInstanceId {
        let mesh_instance_id = self.mesh_instance_counter.next();

        // Add the mesh instance to the right bucket based on its material type.
        match mesh_instance.material_type() {
            &MaterialType::Shared(id) => self.mesh_instances_with_shared_materials.get_mut(&id).unwrap().push(mesh_instance_id),
            &MaterialType::Owned(_) => self.mesh_instances_with_owned_material.push(mesh_instance_id),
        }

        let old = self.mesh_instances.insert(mesh_instance_id, mesh_instance);
        assert!(old.is_none());

        mesh_instance_id
    }

    fn get_mesh_instance(&self, id: MeshInstanceId) -> Option<&MeshInstance> {
        self.mesh_instances.get(&id)
    }

    fn get_mesh_instance_mut(&mut self, id: MeshInstanceId) -> Option<&mut MeshInstance> {
        self.mesh_instances.get_mut(&id)
    }

    fn register_anchor(&mut self, anchor: Anchor) -> AnchorId {
        let anchor_id = self.anchor_counter.next();

        let old = self.anchors.insert(anchor_id, anchor);
        assert!(old.is_none());

        anchor_id
    }

    fn get_anchor(&self, anchor_id: AnchorId) -> Option<&Anchor> {
        self.anchors.get(&anchor_id)
    }

    fn get_anchor_mut(&mut self, anchor_id: AnchorId) -> Option<&mut Anchor> {
        self.anchors.get_mut(&anchor_id)
    }

    fn register_camera(&mut self, camera: Camera) -> CameraId {
        let camera_id = self.camera_counter.next();

        let old = self.cameras.insert(camera_id, camera);
        assert!(old.is_none());

        camera_id
    }

    fn get_camera(&self, camera_id: CameraId) -> Option<&Camera> {
        self.cameras.get(&camera_id)
    }

    fn get_camera_mut(&mut self, camera_id: CameraId) -> Option<&mut Camera> {
        self.cameras.get_mut(&camera_id)
    }

    fn register_light(&mut self, light: Light) -> LightId {
        let light_id = self.light_counter.next();

        let old = self.lights.insert(light_id, light);
        assert!(old.is_none());

        light_id
    }

    fn get_light(&self, light_id: LightId) -> Option<&Light> {
        self.lights.get(&light_id)
    }

    fn get_light_mut(&mut self, light_id: LightId) -> Option<&mut Light> {
        self.lights.get_mut(&light_id)
    }

    fn set_ambient_light(&mut self, color: Color) {
        self.ambient_color = color;
    }
}

unsafe impl Send for GlRender {}

#[derive(Debug)]
pub enum Error {
    ContextError(ContextError),
}

impl From<ContextError> for Error {
    fn from(from: ContextError) -> Error {
        Error::ContextError(from)
    }
}

#[derive(Debug)]
struct MeshData {
    vertex_array: VertexArray,
    position_attribute: VertexAttribute,
    normal_attribute: Option<VertexAttribute>,
    uv_attribute: Option<VertexAttribute>,
    element_count: usize,
}

impl Into<AttribLayout> for VertexAttribute {
    fn into(self) -> AttribLayout {
        AttribLayout {
            elements: self.elements,
            offset: self.offset,
            stride: self.stride,
        }
    }
}
