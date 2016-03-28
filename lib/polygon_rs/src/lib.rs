extern crate bootstrap_rs as bootstrap;
extern crate parse_bmp as bmp;
pub extern crate polygon_math as math;

pub mod camera;
pub mod geometry;
pub mod gl;
pub mod light;
pub mod material;

pub use camera::Camera;
pub use geometry::*;
pub use gl::GlRender;
pub use light::{Light, PointLight};
pub use material::{Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMesh(usize);
