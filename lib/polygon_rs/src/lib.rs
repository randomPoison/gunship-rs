#![feature(question_mark)]

extern crate bootstrap_rs as bootstrap;
extern crate parse_bmp as bmp;

pub extern crate polygon_math as math;

pub mod anchor;
pub mod camera;
pub mod geometry;
pub mod gl;
pub mod light;
pub mod material;
pub mod shader;

use anchor::Anchor;
use camera::Camera;
use geometry::mesh::Mesh;

/// Identifies mesh data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GpuMesh(usize);

impl GpuMesh {
    fn next(&mut self) -> GpuMesh {
        let next = *self;
        self.0 += 1;
        next
    }
}

/// Represents texture data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GpuTexture;

/// Identifies an achor that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AnchorId(usize);

impl AnchorId {
    fn next(&mut self) -> AnchorId {
        let next = *self;
        self.0 += 1;
        next
    }
}

/// Identifies an achor that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CameraId(usize);

impl CameraId {
    fn next(&mut self) -> CameraId {
        let next = *self;
        self.0 += 1;
        next
    }
}

/// The common interface that all rendering systems must provide.
pub trait Renderer {
    /// Renders one frame based on the renderer's current state to the current render target.
    fn draw(&mut self);

    /// Registers mesh data with the renderer, returning a unique id for the mesh.
    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh;

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
}

/// A helper struct for selecting and initializing the most suitable renderer for the client's
/// needs.
pub struct RendererBuilder;

impl RendererBuilder {
    /// Creates a new builder object.
    pub fn new() -> RendererBuilder {
        RendererBuilder
    }

    /// Constructs a new renderer using the options set in the builder.
    pub fn build(&mut self) -> Box<Renderer> {
        Box::new(gl::GlRender::new())
    }
}
