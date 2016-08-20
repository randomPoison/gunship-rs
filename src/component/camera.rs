use component::DefaultManager;
use math::*;

pub type CameraManager = DefaultManager<Camera>;

derive_Component!(Camera);
#[derive(Debug, Clone)]
pub struct Camera
{
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

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
