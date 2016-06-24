pub extern crate gl_util;

use {Counter, GpuMesh, Renderer};
use anchor::*;
use camera::*;
use geometry::mesh::{Mesh, VertexAttribute};
use light::*;
use material::*;
use mesh_instance::*;
use math::*;
use self::gl_util::{
    AttribLayout,
    Comparison,
    DestFactor,
    DrawBuilder,
    DrawMode,
    Face,
    GlMatrix,
    IndexBuffer,
    Program,
    SourceFactor,
    VertexBuffer,
};
use self::gl_util::texture::{
    Texture2d as GlTexture2d,
    TextureFormat,
    TextureInternalFormat,
};
use shader::Shader;
use std::collections::HashMap;
use std::str;
use texture::*;

pub mod shader;

static DEFAULT_SHADER_BYTES: &'static [u8] = include_bytes!("../../resources/shaders/texture_diffuse_lit.shader");

#[derive(Debug)]
pub struct GlRender {
    meshes: HashMap<GpuMesh, MeshData>,
    textures: HashMap<GpuTexture, GlTexture2d>,
    mesh_instances: HashMap<MeshInstanceId, MeshInstance>,
    anchors: HashMap<AnchorId, Anchor>,
    cameras: HashMap<CameraId, Camera>,
    lights: HashMap<LightId, Light>,
    programs: HashMap<Shader, Program>,

    mesh_counter: GpuMesh,
    texture_counter: GpuTexture,
    mesh_instance_counter: MeshInstanceId,
    anchor_counter: AnchorId,
    camera_counter: CameraId,
    light_counter: LightId,
    shader_counter: Shader,

    default_material: Material,
}

impl GlRender {
    pub fn new() -> GlRender {
        gl_util::init();

        // Setup the default program for use in creating the default material.
        let default_shader_source = str::from_utf8(DEFAULT_SHADER_BYTES).unwrap();
        let default_program = shader::from_str(default_shader_source).unwrap();

        let mut shader_counter = Shader::initial();
        let default_shader = shader_counter.next();
        let mut programs = HashMap::new();
        programs.insert(default_shader, default_program);

        let mut default_material = Material::new(default_shader);
        default_material.set_color("surface_color", Color::new(0.25, 0.25, 0.25, 1.0));
        default_material.set_color("surface_specular", Color::new(1.0, 1.0, 1.0, 1.0));
        default_material.set_f32("surface_shininess", 3.0);

        GlRender {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            mesh_instances: HashMap::new(),
            anchors: HashMap::new(),
            cameras: HashMap::new(),
            lights: HashMap::new(),
            programs: programs,

            mesh_counter: GpuMesh::initial(),
            texture_counter: GpuTexture::initial(),
            mesh_instance_counter: MeshInstanceId::initial(),
            anchor_counter: AnchorId::initial(),
            camera_counter: CameraId::initial(),
            light_counter: LightId::initial(),
            shader_counter: shader_counter,

            default_material: default_material,
        }
    }

    fn draw_mesh(
        &self,
        mesh_data: &MeshData,
        material: &Material,
        model_transform: Matrix4,
        normal_transform: Matrix4,
        camera: &Camera,
        camera_anchor: &Anchor,
    ) {
        // Calculate the various transforms needed for rendering.
        let view_transform = camera_anchor.view_matrix();
        let model_view_transform = view_transform * model_transform;
        let projection_transform = camera.projection_matrix();
        let model_view_projection = projection_transform * model_view_transform;

        let view_normal_transform = {
            let inverse_model = normal_transform.transpose();
            let inverse_view = camera_anchor.inverse_view_matrix();
            let inverse_model_view = inverse_model * inverse_view;
            inverse_model_view.transpose()
        };

        let program = self
            .programs
            .get(material.shader())
            .expect("Material is using a shader that does not exist");

        // Set the shader to use.
        let mut draw_builder = DrawBuilder::new(&mesh_data.vertex_buffer, DrawMode::Triangles);
        draw_builder
        .index_buffer(&mesh_data.index_buffer)
        .program(program)
        .cull(Face::Back)
        .depth_test(Comparison::Less)

        // Associate vertex attributes with shader program variables.
        .map_attrib_name("position", "vertex_position")
        .map_attrib_name("normal", "vertex_normal")
        .map_attrib_name("texcoord", "vertex_uv0")

        // Set uniform transforms.
        .uniform(
            "model_transform",
            GlMatrix {
                data: model_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "normal_transform",
            GlMatrix {
                data: view_normal_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "viewTransform",
            GlMatrix {
                data: view_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "model_view_transform",
            GlMatrix {
                data: model_view_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "projectionTransform",
            GlMatrix {
                data: projection_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "model_view_projection",
            GlMatrix {
                data: model_view_projection.raw_data(),
                transpose: true,
            })

        // Set uniform colors.
        .uniform("global_ambient", [0.1, 0.1, 0.1, 1.0])

        // Other uniforms.
        .uniform("cameraPosition", *camera_anchor.position().as_array());

        // Apply material attributes.
        for (name, property) in material.properties() {
            match *property {
                MaterialProperty::Color(ref color) => {
                    draw_builder.uniform::<[f32; 4]>(name, color.into());
                },
                MaterialProperty::F32(value) => {
                    draw_builder.uniform(name, value);
                },
                MaterialProperty::Texture(ref texture) => {
                    let gl_texture = self.textures.get(texture).expect("No such texture exists");
                    draw_builder.uniform(name, gl_texture);
                },
            }
        }

        // Render first light without blending so it overrides any objects behind it.
        // We also render it with light strength 0 so it only renders ambient color.
        draw_builder
            .uniform("light_position", *Point::origin().as_array())
            .uniform("light_strength", 0.0)
            .draw();

        // Render the rest of the lights with blending on the the depth check set to
        // less than or equal.
        draw_builder
            .depth_test(Comparison::LessThanOrEqual)
            .blend(SourceFactor::One, DestFactor::One);

        for light in self.lights.values() {
            // Send the light's position in view space.
            let light_anchor = match light.anchor() {
                Some(anchor_id) => self.anchors.get(&anchor_id).expect("No such anchor exists"),
                None => panic!("Cannot render light if it's not attached to an anchor"),
            };
            let light_position_view = light_anchor.position() * view_transform;
            draw_builder.uniform("light_position", *light_position_view.as_array());

            // Send common light data.
            draw_builder.uniform::<[f32; 4]>("light_color", light.color.into());
            draw_builder.uniform("light_strength", light.strength);

            // Send data specific to the current type of light.
            match light.data {
                LightData::Point(PointLight { radius }) => {
                    draw_builder.uniform("light_radius", radius);
                },
            }

            // Draw the current light.
            draw_builder.draw();
        }
    }

    /// Clears the current back buffer.
    pub fn clear(&self) {
        gl_util::clear();
    }

    /// Swap the front and back buffers for the render system.
    pub fn swap_buffers(&self) {
        gl_util::swap_buffers();
    }
}

impl Renderer for GlRender {
    fn draw(&mut self) {
        self.clear();

        let (camera, camera_anchor) = if let Some(camera) = self.cameras.values().next() {
            // Use the first camera in the scene for now. Eventually we'll want to support
            // rendering multiple cameras to multiple viewports or render targets but for now one
            // is enough.
            let anchor = match camera.anchor() {
                Some(ref anchor_id) => self.anchors.get(anchor_id).expect("no such anchor exists"),
                None => unimplemented!(),
            };

            (camera, anchor)
        } else {
            panic!("There must be a camera registered");
        };

        for mesh_instance in self.mesh_instances.values() {
            let anchor = match mesh_instance.anchor() {
                Some(anchor_id) => self.anchors.get(anchor_id).expect("No such anchor exists"),
                None => continue,
            };

            let model_transform = anchor.matrix();
            let normal_transform = anchor.normal_matrix();

            let mesh = self.meshes.get(mesh_instance.mesh()).expect("Mesh data does not exist for mesh id");

            self.draw_mesh(
                mesh,
                &mesh_instance.material(),
                model_transform,
                normal_transform,
                camera,
                camera_anchor);
        }

        self.swap_buffers();
    }

    fn default_material(&self) -> Material {
        self.default_material.clone()
    }

    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh {
        // Generate array buffer.
        let mut vertex_buffer = VertexBuffer::new();
        vertex_buffer.set_data_f32(mesh.vertex_data());

        // Configure vertex attributes.
        let position = mesh.position();
        vertex_buffer.set_attrib_f32(
            "position",
            AttribLayout {
                elements: position.elements,
                stride: position.stride,
                offset: position.offset,
            });

        if let Some(normal) = mesh.normal() {
            vertex_buffer.set_attrib_f32(
                "normal",
                AttribLayout {
                    elements: normal.elements,
                    stride: normal.stride,
                    offset: normal.offset
                });
        }

        // TODO: Support multiple texcoords.
        if let Some(texcoord) = mesh.texcoord().first() {
            vertex_buffer.set_attrib_f32(
                "texcoord",
                AttribLayout {
                    elements: texcoord.elements,
                    stride: texcoord.stride,
                    offset: texcoord.offset,
                });
        }

        let mut index_buffer = IndexBuffer::new();
        index_buffer.set_data_u32(mesh.indices());

        let mesh_id = self.mesh_counter.next();

        self.meshes.insert(
            mesh_id,
            MeshData {
                vertex_buffer:      vertex_buffer,
                index_buffer:       index_buffer,
                position_attribute: mesh.position(),
                normal_attribute:   mesh.normal(),
                uv_attribute:       None,
                element_count:      mesh.indices().len(),
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
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8(ref data) => {
                GlTexture2d::new(
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8x3(ref data) => {
                GlTexture2d::new(
                    format,
                    internal_format,
                    texture.width(),
                    texture.height(),
                    &*data)
            },
            &TextureData::u8x4(ref data) => {
                GlTexture2d::new(
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
}

#[derive(Debug)]
struct MeshData {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: Option<VertexAttribute>,
    pub uv_attribute: Option<VertexAttribute>,
    element_count: usize,
}
