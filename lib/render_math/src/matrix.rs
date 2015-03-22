use std::cmp::{PartialEq, Eq};
use std::ops::{Index, IndexMut, Mul};
use std::num::Float;

/// A 4x4 matrix that can be used to transform 3D points and vectors.
///
/// Matrices are row-major.
#[repr(C)] #[derive(Copy)] #[derive(Debug)]
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

    pub fn from_rotation(x: f32, y: f32, z: f32) -> Matrix4 {
        let x_rot = Matrix4 {
            data: [
                1.0, 0.0,      0.0,     0.0,
                0.0, x.cos(), -x.sin(), 0.0,
                0.0, x.sin(),  x.cos(), 0.0,
                0.0, 0.0,      0.0,     1.0
            ]
        };

        let y_rot = Matrix4 {
            data: [
                 y.cos(), 0.0, y.sin(), 0.0,
                 0.0,     1.0, 0.0,     0.0,
                -y.sin(), 0.0, y.cos(), 0.0,
                 0.0,     0.0, 0.0,     1.0
            ]
        };

        let z_rot = Matrix4 {
            data: [
                z.cos(), -z.sin(), 0.0, 0.0,
                z.sin(),  z.cos(), 0.0, 0.0,
                0.0,      0.0,     1.0, 0.0,
                0.0,      0.0,     0.0, 1.0
            ]
        };

        z_rot * (y_rot * x_rot)
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

impl PartialEq for Matrix4 {
    fn ne(&self, other: &Matrix4) -> bool {
        for (&ours, &theirs) in self.data.iter().zip(other.data.iter()) {
            if ours != theirs {
                return true
            }
        }
        false
    }

    fn eq(&self, other: &Matrix4) -> bool {
        !(self != other)
    }
}

impl Eq for Matrix4 {}

impl Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index<'a>(&'a self, index: &(usize, usize)) -> &'a f32 {
        let &(row, col) = index;
        assert!(row < 4 && col < 4);
        &self.data[row * 4 + col]
    }
}

impl IndexMut<(usize, usize)> for Matrix4 {
    fn index_mut<'a>(&'a mut self, index: &(usize, usize)) -> &'a mut f32 {
        let &(row, col) = index;
        assert!(row < 4 && col < 4);
        &mut self.data[row * 4 + col]
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, other: Matrix4) -> Matrix4 {
        let mut result = Matrix4::new();

        // TODO: Should this be written with iterators instead?
        for row in 0..4 {
            for col in 0..4 {
                result[(row, col)] = {
                    let mut dot_product = 0.0;
                    for offset in 0..4 {
                        dot_product +=
                            self[(row, offset)] *
                            other[(offset, col)];
                    }
                    dot_product
                };
            }
        }

        result
    }
}
