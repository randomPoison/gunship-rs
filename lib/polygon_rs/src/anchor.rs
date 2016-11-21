use math::*;

#[derive(Debug)]
pub struct Anchor {
    position: Point,
    orientation: Orientation,
    scale: Vector3,
}

impl Anchor {
    /// Creates a new anchor.
    pub fn new() -> Anchor {
        Anchor {
            position: Point::origin(),
            orientation: Orientation::new(),
            scale: Vector3::one(),
        }
    }

    /// Gets the current position of the anchor.
    pub fn position(&self) -> Point {
        self.position
    }

    /// Sets the position of the anchor.
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    /// Gets the current orientation of the anchor.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Sets the orientation of the anchor.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    /// Gets the current scale of the anchor.
    pub fn scale(&self) -> Vector3 {
        self.scale
    }

    /// Sets the scale of the anchor.
    pub fn set_scale(&mut self, scale: Vector3) {
        self.scale = scale;
    }

    /// Calculates the matrix to convert from object space to world space.
    pub fn matrix(&self) -> Matrix4 {
        let position = Matrix4::from_point(self.position);
        let orientation = Matrix4::from(self.orientation);
        let scale = Matrix4::from_scale_vector(self.scale);

        position * (orientation * scale)
    }

    /// Calculates the matrix used to convert normals from object space to world space.
    pub fn normal_matrix(&self) -> Matrix3 {
        let inv_scale = Matrix3::from_scale_vector(1.0 / self.scale);
        let orientation: Matrix3 = self.orientation.into();
        let inv_rotation = orientation.transpose();
        // let inv_translation = Matrix3::from_point(-self.position);

        let inverse = inv_scale * (inv_rotation);// * inv_translation);
        inverse.transpose()
    }

    /// Calculates the view transform for the camera.
    ///
    /// The view transform the matrix that converts from world coordinates to camera coordinates.
    pub fn view_matrix(&self) -> Matrix4 {
        let inv_orientation = Matrix4::from(self.orientation).transpose();
        let inv_translation = Matrix4::translation(
            -self.position.x,
            -self.position.y,
            -self.position.z);
        inv_orientation * inv_translation
    }

    /// Calculates the inverse view matrix.
    pub fn inverse_view_matrix(&self) -> Matrix4 {
        Matrix4::from_point(self.position) * self.orientation.into()
    }
}

/// Identifies an achor that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AnchorId(usize);
derive_Counter!(AnchorId);
