use async::engine::{self, EngineMessage};
use async::transform::Transform;
use std::f32::consts::PI;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Camera {
    data: *mut CameraData,
    _phantom: PhantomData<CameraData>,
}

impl Camera {
    pub fn new(transform: &Transform) -> Camera {
        let mut camera_data = Box::new(CameraData::default());

        let ptr = &mut *camera_data as *mut _;

        engine::send_message(EngineMessage::Camera(camera_data, transform.inner()));

        Camera {
            data: ptr,
            _phantom: PhantomData,
        }
    }
}

impl Deref for Camera {
    type Target = CameraData;

    fn deref(&self) -> &CameraData { unsafe { &*self.data } }
}

impl DerefMut for Camera {
    fn deref_mut(&mut self) -> &mut CameraData { unsafe { &mut *self.data } }
}

#[derive(Debug)]
pub struct CameraData {
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl CameraData {
    pub fn fov(&self) -> f32 { self.fov }

    pub fn aspect(&self) -> f32 { self.aspect }

    pub fn near(&self) -> f32 { self.near }

    pub fn far(&self) -> f32 { self.far }
}

impl Default for CameraData {
    fn default() -> CameraData {
        CameraData {
            fov: PI / 3.0,
            aspect: 1.0,
            near: 0.001,
            far: 1_000.0,
        }
    }
}
