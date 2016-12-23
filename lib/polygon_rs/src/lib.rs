extern crate bootstrap_rs as bootstrap;
extern crate parse_bmp;
extern crate polygon_material;
extern crate stopwatch;

pub extern crate polygon_math as math;

// NOTE: This is a "standard" workaround for Rust's nasty macro visibility rules. Once the new
// macro system arrives this can be removed.
#[macro_use]
mod macros;

pub mod anchor;
pub mod camera;
pub mod geometry;
pub mod gl;
pub mod light;
pub mod material;
pub mod mesh_instance;
pub mod shader;
pub mod texture;

use anchor::*;
use bootstrap::window::Window;
use camera::*;
use geometry::mesh::Mesh;
use light::*;
use material::*;
use math::Color;
use mesh_instance::*;
use texture::*;

/// Identifies mesh data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GpuMesh(usize);
derive_Counter!(GpuMesh);

/// The common interface that all rendering systems must provide.
pub trait Renderer: 'static + Send {
    /// Renders one frame based on the renderer's current state to the current render target.
    fn draw(&mut self);

    /// Gets a copy of the default material for the renderer.
    fn default_material(&self) -> Material;

    /// Parses a material source file and generates a material from it.
    fn build_material(&mut self, source: MaterialSource) -> Result<Material, BuildMaterialError>;

    /// Registers a material to be used as a shared material.
    fn register_shared_material(&mut self, material: Material) -> MaterialId;

    /// Gets a registered material.
    fn get_material(&self, material_id: MaterialId) -> Option<&Material>;

    /// Registers mesh data with the renderer, returning a unique id for the mesh.
    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh;

    /// Registers texture data with the renderer, returning a unique id for the texture.
    fn register_texture(&mut self, texture: &Texture2d) -> GpuTexture;

    /// Registers a mesh instance with the renderer, returning a unique id for that mesh instance.
    fn register_mesh_instance(&mut self, mesh_instance: MeshInstance) -> MeshInstanceId;

    /// Gets a reference to a registered mesh instance.
    fn get_mesh_instance(&self, id: MeshInstanceId) -> Option<&MeshInstance>;

    /// Gets a mutable reference to a registered mesh instance.
    fn get_mesh_instance_mut(&mut self, id: MeshInstanceId) -> Option<&mut MeshInstance>;

    /// Registers an anchor with the renderer, returning a unique id for the anchor.
    fn register_anchor(&mut self, anchor: Anchor) -> AnchorId;

    /// Gets a reference to a registered anchor.
    fn get_anchor(&self, anchor_id: AnchorId) -> Option<&Anchor>;

    /// Gets a mutable reference to a registered anchor.
    fn get_anchor_mut(&mut self, anchor_id: AnchorId) -> Option<&mut Anchor>;

    /// Registers a camera with the renderer, returning a unique id for the camera.
    fn register_camera(&mut self, camera: Camera) -> CameraId;

    /// Gets a reference to a registered camera.
    fn get_camera(&self, camera_id: CameraId) -> Option<&Camera>;

    /// Gets a mutable reference to a registered camera.
    fn get_camera_mut(&mut self, camera_id: CameraId) -> Option<&mut Camera>;

    /// Registers a light with the renderer, returning a unique id for the light.
    fn register_light(&mut self, light: Light) -> LightId;

    /// Gets a reference to a registered light.
    fn get_light(&self, light_id: LightId) -> Option<&Light>;

    /// Gets a mutable reference to a registered light.
    fn get_light_mut(&mut self, light_id: LightId) -> Option<&mut Light>;

    fn set_ambient_light(&mut self, color: Color);
}

/// A helper struct for selecting and initializing the most suitable renderer for the client's
/// needs.
pub struct RendererBuilder<'a> {
    window: &'a Window,
}

impl<'a> RendererBuilder<'a> {
    /// Creates a new builder object.
    pub fn new(window: &Window) -> RendererBuilder {
        RendererBuilder {
            window: window,
        }
    }

    /// Constructs a new renderer using the options set in the builder.
    pub fn build(&mut self) -> Box<Renderer> {
        let renderer = gl::GlRender::new(self.window).unwrap();
        Box::new(renderer) as Box<Renderer>
    }
}

/// Extra special secret trait for keep counter functionality local to this crate.
///
/// All resources managed by a renderer have an associated ID type used to reference the data
/// owned by the renderer. The renderers internally keep a counter to generate new ID values, but
/// the functionality for creating new ID values needs to be kept private to polygon. In order to
/// avoid having to define all ID types at the root of the crate (which would give the renderers
/// access to the private methods to create new ID values) we define the private `Counter` trait
/// and implement it for all ID types using the `derive_Counter!` macro. This allows us to define
/// various ID types in the most appropriate module while still giving all renderers the ability
/// to create new ID values.
trait Counter {
    /// Creates a new counter with the initial value.
    fn initial() -> Self;

    /// Returns the next valid ID value, updating the internal counter in the process.
    fn next(&mut self) -> Self;
}

#[derive(Debug)]
pub struct BuildMaterialError;
