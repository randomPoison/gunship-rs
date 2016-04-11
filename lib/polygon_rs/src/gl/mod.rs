pub extern crate gl_util;

use {AnchorId, GpuMesh, GpuTexture};
use anchor::*;
use bmp::Bitmap;
use camera::Camera;
use geometry::mesh::{Mesh, VertexAttribute};
use light::Light;
use material::*;
use math::*;
use Renderer;
use std::collections::HashMap;

pub use self::gl_util::*;

#[derive(Debug)]
pub struct GlRender {
    meshes: HashMap<GpuMesh, MeshData>,
    anchors: HashMap<AnchorId, Anchor>,

    mesh_counter: usize,
    anchor_counter: usize,
}

impl GlRender {
    pub fn new() -> GlRender {
        gl_util::init();

        GlRender {
            meshes: HashMap::new(),
            anchors: HashMap::new(),

            mesh_counter: 0,
            anchor_counter: 0,
        }
    }

    fn gen_texture(&self, _bitmap: &Bitmap) -> GpuTexture {
        unimplemented!();
    }

    fn draw_mesh(
        &self,
        mesh_data: &MeshData,
        material: &Material,
        model_transform: Matrix4,
        normal_transform: Matrix4,
        camera: &Camera,
        lights: &mut Iterator<Item=Light>
    ) {
        // Calculate the various transforms neede for rendering.
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
        let mut draw_builder = DrawBuilder::new(&mesh_data.vertex_buffer, DrawMode::Triangles);
        draw_builder
        .index_buffer(&mesh_data.index_buffer)
        .program(material.shader())

        // Associate vertex attributes with shader program variables.
        .map_attrib_name("position", "vertexPosition")
        .map_attrib_name("normal", "vertexNormal")

        // Set uniform transforms.
        .uniform(
            "modelTransform",
            GlMatrix {
                data: model_transform.raw_data(),
                transpose: true,
            })
        .uniform(
            "normalTransform",
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
            "modelViewTransform",
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
            "modelViewProjection",
            GlMatrix {
                data: model_view_projection.raw_data(),
                transpose: true,
            })

        // Set uniform colors.
        .uniform("globalAmbient", *Color::new(0.25, 0.25, 0.25, 1.0).as_array())

        // Other uniforms.
        .uniform("cameraPosition", *camera.position.as_array());

        // // Apply material attributes.
        // for (name, property) in material.properties() {
        //     match *property {
        //         MaterialProperty::Color(color) => {
        //
        //         },
        //         MaterialProperty::Texture(ref texture) => {
        //
        //         },
        //     }
        // }

        // Shader doesn't accept lights, so just draw without setting light values.
        draw_builder.draw();

        // if let Some(light_position_location) = shader.light_position {
        //     // Render first light without blending so it overrides any objects behind it.
        //     // We also render it with light strength 0 so it only renders ambient color.
        //     {
        //         gl::uniform_4f(light_position_location, Point::origin().as_array());
        //         if let Some(light_strength_location) = shader.light_strength {
        //             gl::uniform_1f(light_strength_location, 0.0);
        //         }
        //         gl::disable(ServerCapability::Blend);
        //         gl::draw_elements(
        //             DrawMode::Triangles,
        //             mesh.element_count as i32,
        //             IndexType::UnsignedInt,
        //             0);
        //     }
        //
        //     // Render the rest of the lights with blending on the the depth check set to LEQUAL.
        //     gl::depth_func(Comparison::LEqual);
        //     gl::enable(ServerCapability::Blend);
        //     gl::blend_func(SourceFactor::One, DestFactor::One);
        //
        //     let ambient_color = Color::new(0.0, 0.0, 0.0, 1.0);
        //     if let Some(ambient_location) = shader.global_ambient {
        //         gl::uniform_4f(ambient_location, ambient_color.as_array());
        //     }
        //
        //     for light in lights {
        //         let light_position = match light {
        //             Light::Point(ref point_light) => point_light.position
        //         };
        //         let light_position_view = light_position * view_transform;
        //
        //         gl::uniform_4f(light_position_location, light_position_view.as_array());
        //         if let Some(light_strength_location) = shader.light_strength {
        //             // TODO: Use the light's actual strength value.
        //             gl::uniform_1f(light_strength_location, 1.0);
        //         }
        //
        //         // TODO: Set uniforms and all that jazz.
        //         DrawBuilder::new(mesh.vertex_buffer, DrawMode::Triangles)
        //         .index_buffer(mesh.index_buffer)
        //         .draw();
        //     }
        // }
    }

    pub fn draw_wireframe(
        &self,
        camera: &Camera,
        material: &Material,
        mesh: &GpuMesh,
        model_transform: Matrix4,
        color: Color,
    ) {
        /*
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
        */

        // if let Some(surface_color_location) = shader.surface_color {
        //     gl::uniform_4f(surface_color_location, color.as_array());
        // }

        // Get mesh data.
        let mesh_data = self.meshes.get(mesh).expect("No such gpu mesh exists");

        DrawBuilder::new(&mesh_data.vertex_buffer, DrawMode::Lines)
        .index_buffer(&mesh_data.index_buffer)
        .draw();
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

        for anchor in self.anchors.values() {
            for mesh_id in anchor.meshes() {
                let mesh = self.meshes.get(mesh_id).expect("Mesh data does not exist for mesh id");

                self.draw_mesh(
                    mesh,
                    &Material::default(),
                    Matrix4::identity(),
                    Matrix4::identity(),
                    &Camera::default(),
                    &mut None.into_iter() as &mut Iterator<Item=Light>);
            }

        }

        self.swap_buffers();
    }

    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh {
        // Generate array buffer.
        let mut vertex_buffer = VertexBuffer::new();
        vertex_buffer.set_data_f32(mesh.vertex_data());

        // Configure vertex attributes.
        let position = mesh.position();
        vertex_buffer.set_attrib_f32("position", 4, position.stride, position.offset);

        if let Some(normal) = mesh.normal() {
            vertex_buffer.set_attrib_f32("normal", 3, normal.stride, normal.offset);
        }

        let mut index_buffer = IndexBuffer::new();
        index_buffer.set_data_u32(mesh.indices());

        let mesh_id = self.mesh_counter;
        self.mesh_counter += 1;

        self.meshes.insert(
            GpuMesh(mesh_id),
            MeshData {
                vertex_buffer:      vertex_buffer,
                index_buffer:       index_buffer,
                position_attribute: mesh.position(),
                normal_attribute:   mesh.normal(),
                uv_attribute:       None,
                element_count:      mesh.indices().len(),
            });

        GpuMesh(mesh_id)
    }

    fn register_anchor(&mut self, anchor: Anchor) -> AnchorId {
        let anchor_id = self.anchor_counter;
        self.anchor_counter += 1;

        let old = self.anchors.insert(AnchorId(anchor_id), anchor);
        assert!(old.is_none());

        AnchorId(anchor_id)
    }

    fn get_anchor(&self, anchor_id: AnchorId) -> Option<&Anchor> {
        self.anchors.get(&anchor_id)
    }

    fn get_anchor_mut(&mut self, anchor_id: AnchorId) -> Option<&mut Anchor> {
        self.anchors.get_mut(&anchor_id)
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
