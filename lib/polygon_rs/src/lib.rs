extern crate polygon_math as math;
extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_gl as gl;

pub mod geometry;
pub mod gl_render;
pub mod camera;
pub mod light;

pub use camera::Camera;
pub use light::{Light, PointLight};
pub use gl_render::{GLRender, ShaderProgram};
