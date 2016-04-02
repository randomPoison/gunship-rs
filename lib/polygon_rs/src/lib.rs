#![feature(question_mark)]

extern crate bootstrap_rs as bootstrap;
extern crate parse_bmp as bmp;

pub extern crate polygon_math as math;

pub mod camera;
pub mod geometry;
pub mod gl;
pub mod light;
pub mod material;
pub mod shader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMesh(usize);

use geometry::mesh::Mesh;

/// The common interface that all rendering systems must provide.
pub trait Renderer {
    /// Render one frame based on the renderer's current state to the current render target
    /// (usually the screen).
    fn draw(&mut self);

    /// Register mesh data with the renderer, allowing it to format and send that data to the GPU.
    fn register_mesh(&mut self, mesh: &Mesh);
}
