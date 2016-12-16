use engine::{self, EngineMessage};
use transform::Transform;
use std::f32::consts::PI;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::Unique;

pub struct Camera {
    data: Unique<CameraData>,

    // Pretend `Camera` owns a raw pointer to default implementation for `Sync`.
    // TODO: Remove this once negative trait bounds are stabilized.
    _phantom: PhantomData<*mut ()>,
}

impl Camera {
    pub fn new(transform: &Transform) -> Camera {
        let mut camera_data = Box::new(CameraData::default());

        let ptr = &mut *camera_data as *mut _;

        engine::send_message(EngineMessage::Camera(camera_data, transform.inner()));

        Camera {
            data: unsafe { Unique::new(ptr) },
            _phantom: PhantomData,
        }
    }

    pub fn forget(self) {
        mem::forget(self);
    }
}

unsafe impl Send for Camera {}

impl Debug for Camera {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        let data = unsafe { self.data.get() };

        fmt.debug_struct("Camera")
            .field("fov", &data.fov)
            .field("aspect", &data.aspect)
            .field("near", &data.near)
            .field("far", &data.far)
            .finish()
    }
}

impl Deref for Camera {
    type Target = CameraData;

    fn deref(&self) -> &CameraData { unsafe { self.data.get() } }
}

impl DerefMut for Camera {
    fn deref_mut(&mut self) -> &mut CameraData { unsafe { self.data.get_mut() } }
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
