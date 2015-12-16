use component::DefaultManager;
use ecs::*;
use math::*;
use std::f32::consts::PI;

pub type CameraManager = DefaultManager<Camera>;

#[derive(Debug, Clone)]
pub struct Camera
{
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn to_polygon_camera(&self, position: Point, orientation: Quaternion) -> ::polygon::camera::Camera {
        ::polygon::camera::Camera {
            fov: self.fov,
            aspect: self.aspect,
            near: self.near,
            far: self.far,

            position: position,
            rotation: orientation,
        }
    }
}

impl Component for Camera {}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            fov:    PI / 3.0,
            aspect: 1.0,
            near:   0.001,
            far:    100.0,
        }
    }
}
