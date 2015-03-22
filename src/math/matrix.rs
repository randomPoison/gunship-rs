/// A 4x4 matrix that can be used to transform 3D points and vectors.
///
/// Matrices are row-major.
#[repr(C)] #[derive(Copy)]
pub struct Matrix4 {
    data: [f32; 16]
}

impl Matrix4 {

    /// Create a new identity matrix.
    pub fn new() -> Matrix4 {
        Matrix4 {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ]
        }
    }

    /// Create a new translation matrix.
    pub fn from_translation(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4 {
            data: [
                1.0, 0.0, 0.0, x,
                0.0, 1.0, 0.0, y,
                0.0, 0.0, 1.0, z,
                0.0, 0.0, 0.0, 1.0
            ]
        }
    }

    /// Get the matrix data as a raw array.
    ///
    /// This is meant to be used for ffi and when passing
    /// matrix data to the graphics card, it should not
    /// be used to directly manipulate the contents of the matrix.
    pub unsafe fn raw_data(&self) -> *const f32
    {
        &self.data[0]
    }
}
