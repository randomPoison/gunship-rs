use math::Point;
use math::Matrix4;
use math::Quaternion;

/// A camera in the scene.
#[derive(Debug, Copy, Clone)]
pub struct Camera
{
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub position: Point,
    pub rotation: Quaternion,
}

impl Camera
{
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Camera {
        Camera {
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,

            position: Point::origin(),
            rotation: Quaternion::identity(),
        }
    }

    /// Calculates the view transform for the camera.
    ///
    /// The view transform the matrix that converts from world coordinates
    /// to camera coordinates.
    pub fn view_matrix(&self) -> Matrix4 {
        self.rotation.as_matrix4().transpose() * Matrix4::translation(-self.position.x, -self.position.y, -self.position.z)
    }

    pub fn inverse_view_matrix(&self) -> Matrix4 {
        Matrix4::from_point(self.position) * self.rotation.as_matrix4()
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
}
