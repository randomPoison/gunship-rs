extern crate bootstrap_gl as gl;
extern crate bootstrap_rs as bootstrap;
extern crate parse_bmp as bmp;
extern crate polygon_math as math;

pub mod camera;
pub mod geometry;
pub mod gl_render;
pub mod light;

pub use camera::Camera;
pub use geometry::*;
pub use gl_render::{GLRender, ShaderProgram};
pub use light::{Light, PointLight};
