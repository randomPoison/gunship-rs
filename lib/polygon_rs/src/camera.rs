use anchor::AnchorId;
use math::*;

/// A camera in the scene.
#[derive(Debug, Clone)]
pub struct Camera
{
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    anchor: Option<AnchorId>,
}

impl Camera
{
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Camera {
        Camera {
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,

            anchor: None,
        }
    }

    /// Calculates the projection matrix for the camera.
    ///
    /// The projection matrix is the matrix that converts from camera space to
    /// clip space. This effectively converts the viewing frustrum into a unit cube.
    pub fn projection_matrix(&self) -> Matrix4 {
        let height = 2.0 * self.near * (self.fov * 0.5).tan();
        let width = self.aspect * height;

        let mut projection = Matrix4::new();
        projection[0][0] = 2.0 * self.near / width;
        projection[1][1] = 2.0 * self.near / height;
        projection[2][2] = -(self.far + self.near) / (self.far - self.near);
        projection[2][3] = -2.0 * self.far * self.near / (self.far - self.near);
        projection[3][2] = -1.0;
        projection
    }

    pub fn anchor(&self) -> Option<AnchorId> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor_id: AnchorId) {
        self.anchor = Some(anchor_id);
    }

    pub fn set_fov(&mut self, fov: f32) {
        debug_assert!(fov > 0.0, "Field of view must be non-negative: {}", fov);
        debug_assert!(fov < PI * 2.0, "Field of view must be less than 180 degrees: {}", fov);
        self.fov = fov;
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        debug_assert!(aspect > 0.0, "Aspect ratio must be non-negative: {}", aspect);
        self.aspect = aspect;
    }

    pub fn set_near(&mut self, near: f32) {
        debug_assert!(near > 0.0, "Near plane distance must be non-negative: {}", near);
        debug_assert!(near < self.far, "Near plane distance must be less than far plane distance, near: {}, far: {}", near, self.far);
        self.near = near;
    }

    pub fn set_far(&mut self, far: f32) {
        debug_assert!(far > 0.0, "Far plane distance must be non-negative: {}", far);
        debug_assert!(far > self.near, "Far plane distance must be greater than near plane distance, near: {}, far: {}", self.near, far);
        self.far = far;
    }
}

impl Default for Camera {
    /// Creates a new
    fn default() -> Camera {
        Camera {
            fov: PI / 3.0,
            aspect: 1.0,
            near: 0.001,
            far: 1_000.0,

            anchor: None,
        }
    }
}

/// Identifies an achor that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CameraId(usize);
derive_Counter!(CameraId);
