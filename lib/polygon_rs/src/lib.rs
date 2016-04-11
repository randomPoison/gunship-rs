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
use geometry::mesh::Mesh;

/// Identifies mesh data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMesh(usize);

/// Represents texture data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuTexture;

/// Identifies an achor that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnchorId(usize);

/// The common interface that all rendering systems must provide.
pub trait Renderer {
    /// Render one frame based on the renderer's current state to the current render target
    /// (usually the screen).
    fn draw(&mut self);

    /// Registers mesh data with the renderer, returning a unique id for the mesh.
    fn register_mesh(&mut self, mesh: &Mesh) -> GpuMesh;

    /// Registers an anchor with the renderer, returning a unique id for the anchor.
    fn register_anchor(&mut self, anchor: Anchor) -> AnchorId;

    /// Gets a reference to a registered anchor.
    fn get_anchor(&self, anchor_id: AnchorId) -> Option<&Anchor>;

    /// Gets a mutable reference to a registered anchor.
    fn get_anchor_mut(&mut self, anchor_id: AnchorId) -> Option<&mut Anchor>;
}

/// A helper struct for selecting and initializing the most suitable renderer for the client's
/// needs.
pub struct RendererBuilder;

impl RendererBuilder {
    pub fn new() -> RendererBuilder {
        RendererBuilder
    }

    pub fn build(&mut self) -> Box<Renderer> {
        Box::new(gl::GlRender::new())
    }
}
