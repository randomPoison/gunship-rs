pub extern crate gl_util;

use {Counter, GpuMesh, Renderer};
use anchor::*;
use camera::*;
use geometry::mesh::{Mesh, VertexAttribute};
use light::*;
use material::*;
use mesh_instance::*;
use math::*;
use std::collections::HashMap;

pub use self::gl_util::*;

#[derive(Debug)]
pub struct GlRender {
    meshes: HashMap<GpuMesh, MeshData>,
    mesh_instances: HashMap<MeshInstanceId, MeshInstance>,
    anchors: HashMap<AnchorId, Anchor>,
    cameras: HashMap<CameraId, Camera>,
    lights: HashMap<LightId, Light>,

    mesh_counter: GpuMesh,
    mesh_instance_counter: MeshInstanceId,
    anchor_counter: AnchorId,
    camera_counter: CameraId,
    light_counter: LightId,
}

impl GlRender {
    pub fn new() -> GlRender {
        gl_util::init();

        GlRender {
            meshes: HashMap::new(),
            mesh_instances: HashMap::new(),
            anchors: HashMap::new(),
            cameras: HashMap::new(),
            lights: HashMap::new(),

            mesh_counter: GpuMesh::initial(),
            mesh_instance_counter: MeshInstanceId::initial(),
            anchor_counter: AnchorId::initial(),
            camera_counter: CameraId::initial(),
            light_counter: LightId::initial(),
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

        // Set the shader to use.
        let mut draw_builder = DrawBuilder::new(&mesh_data.vertex_buffer, DrawMode::Triangles);
        draw_builder
        .index_buffer(&mesh_data.index_buffer)
        .program(material.shader())
        .cull(Face::Back)
        .depth_test(Comparison::Less)

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
        .uniform("surfaceDiffuse", *Color::new(0.25, 0.25, 0.25, 1.0).as_array())

        // Other uniforms.
        .uniform("cameraPosition", *camera_anchor.position().as_array());

        // Apply material attributes.
        for (name, property) in material.properties() {
            match *property {
                MaterialProperty::Color(ref color) => {
                    draw_builder.uniform(name, *color.as_array());
                },
                MaterialProperty::Texture(ref _texture) => {
                    unimplemented!();
                },
            }
        }

        // Render first light without blending so it overrides any objects behind it.
        // We also render it with light strength 0 so it only renders ambient color.
        draw_builder
            .uniform("lightPosition", *Point::origin().as_array())
            .uniform("lightStrength", 0.0)
            .draw();

        // Render the rest of the lights with blending on the the depth check set to
        // less than or equal.
        let ambient_color = Color::new(0.0, 0.0, 0.0, 1.0);
        draw_builder
            .depth_test(Comparison::LessThanOrEqual)
            .blend(SourceFactor::One, DestFactor::One)
            .uniform("ambientLocation", *ambient_color.as_array());

        for light in self.lights.values() {
            // Send the light's position in view space.
            let light_anchor = match light.anchor() {
                Some(anchor_id) => self.anchors.get(&anchor_id).expect("No such anchor exists"),
                None => panic!("Cannot render light if it's not attached to an anchor"),
            };
            let light_position_view = light_anchor.position() * view_transform;
            draw_builder.uniform("lightPosition", *light_position_view.as_array());

            // Send common light data.
            draw_builder.uniform("lightColor", *light.color.as_array());
            draw_builder.uniform("lightStrength", light.strength);

            // Send data specific to the current type of light.
            match light.data {
                LightData::Point(PointLight { radius }) => {
                    draw_builder.uniform("lightRadius", radius);
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
                &Material::default(),
                model_transform,
                normal_transform,
                camera,
                camera_anchor);
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
