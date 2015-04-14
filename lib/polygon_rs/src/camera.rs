use math::point::Point;
use math::vector::Vector3;
use math::matrix::Matrix4;

/// A camera in the scene.
pub struct Camera
{
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub position: Point,
    pub rotation: Matrix4
}

impl Camera
{
    /// Recalculates the rotation of the camera so that it looks at the given point.
    pub fn look_at(&mut self, interest: Point, up: Vector3) {
        let forward = interest - self.position;
        let forward = forward.normalized();
        let up = up.normalized();

        let right = Vector3::cross(forward, up);
        let up = Vector3::cross(right, forward);

        let mut look_matrix = Matrix4::identity();

        look_matrix[(0, 0)] = right.x;
        look_matrix[(1, 0)] = right.y;
        look_matrix[(2, 0)] = right.z;

        look_matrix[(0, 1)] = up.x;
        look_matrix[(1, 1)] = up.y;
        look_matrix[(2, 1)] = up.z;

        look_matrix[(0, 2)] = -forward.x;
        look_matrix[(1, 2)] = -forward.y;
        look_matrix[(2, 2)] = -forward.z;

        self.rotation = look_matrix;
    }

    /// Calculates the view transform for the camera.
    ///
    /// The view transform the matrix that converts from world coordinates
    /// to camera coordinates.
    pub fn view_transform(&self) -> Matrix4 {
        self.rotation.transpose() * Matrix4::from_translation(-self.position.x, -self.position.y, -self.position.z)
    }

    /// Calculates the projection matrix for the camera.
    ///
    /// The projection matrix is the matrix that converts from camera space to
    /// clip space. This effectively converts the viewing frustrum into a unit cube.
    pub fn projection_matrix(&self) -> Matrix4 {
        let height = 2.0 * self.near * (self.fov * 0.5).tan();
        let width = self.aspect * height;

        let mut projection = Matrix4::new();
        projection[(0, 0)] = 2.0 * self.near / width;
        projection[(1, 1)] = 2.0 * self.near / height;
        projection[(2, 2)] = -(self.far + self.near) / (self.far - self.near);
        projection[(2, 3)] = -2.0 * self.far * self.near / (self.far - self.near);
        projection[(3, 2)] = -1.0;
        projection
    }
}
