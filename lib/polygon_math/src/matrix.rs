use std::ops::{Index, IndexMut, Mul};
use std::fmt::{Debug, Formatter, Error};
use std::cmp::PartialEq;

use vector::Vector3;
use point::Point;
use quaternion::Quaternion;

pub const EPSILON: f32 = 0.00001;

/// A 4x4 matrix that can be used to transform 3D points and vectors.
///
/// Matrices are row-major.
#[repr(C)] #[derive(Clone, Copy)]
pub struct Matrix4 {
    data: [f32; 16]
}

impl Matrix4 {
    /// Create a new empy matrix.
    ///
    /// The result matrix is filled entirely with zeroes, it is NOT an identity
    /// matrix. use Matrix4::identity() to get a new identit matrix.
    pub fn new() -> Matrix4 {
        Matrix4 {
            data: [
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0
            ]
        }
    }

    /// Create a new identity matrix.
    pub fn identity() -> Matrix4 {
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
    pub fn translation(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4 {
            data: [
                1.0, 0.0, 0.0, x,
                0.0, 1.0, 0.0, y,
                0.0, 0.0, 1.0, z,
                0.0, 0.0, 0.0, 1.0
            ]
        }
    }

    pub fn from_point(point: Point) -> Matrix4 {
        Matrix4::translation(point.x, point.y, point.z)
    }

    pub fn rotation(x: f32, y: f32, z: f32) -> Matrix4 {
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

    pub fn from_quaternion(q: &Quaternion) -> Matrix4 {
        Matrix4 {
            data: [
                (q.w*q.w + q.x*q.x - q.y*q.y - q.z*q.z), (2.0*q.x*q.y - 2.0*q.w*q.z),             (2.0*q.x*q.z + 2.0*q.w*q.y),             0.0,
                (2.0*q.x*q.y + 2.0*q.w*q.z),             (q.w*q.w - q.x*q.x + q.y*q.y - q.z*q.z), (2.0*q.y*q.z - 2.0*q.w*q.x),             0.0,
                (2.0*q.x*q.z - 2.0*q.w*q.y),             (2.0*q.y*q.z + 2.0*q.w*q.x),             (q.w*q.w - q.x*q.x - q.y*q.y + q.z*q.z), 0.0,
                0.0,                                     0.0,                                     0.0,                                     1.0,
            ]
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4 {
            data: [
                x,   0.0, 0.0, 0.0,
                0.0, y,   0.0, 0.0,
                0.0, 0.0, z,   0.0,
                0.0, 0.0, 0.0, 1.0
            ]
        }
    }

    pub fn transpose(&self) -> Matrix4 {
        let mut transpose = *self;
        for row in 0..4 {
            for col in (row + 1)..4
            {
                let temp = transpose[(row, col)];
                transpose[(row, col)] = transpose[(col, row)];
                transpose[(col, row)] = temp;
            }
        }
        transpose
    }

    pub fn x_part(&self) -> Vector3 {
        Vector3::new(self[(0, 0)], self[(1, 0)], self[(2, 0)])
    }

    pub fn y_part(&self) -> Vector3 {
        Vector3::new(self[(0, 1)], self[(1, 1)], self[(2, 1)])
    }

    pub fn z_part(&self) -> Vector3 {
        Vector3::new(self[(0, 2)], self[(1, 2)], self[(2, 2)])
    }

    pub fn translation_part(&self) -> Point {
        Point::new(self[(0, 3)], self[(1, 3)], self[(2, 3)])
    }

    /// Get the matrix data as a raw array.
    ///
    /// This is meant to be used for ffi and when passing matrix data to the graphics card,
    /// it should not be used to directly manipulate the contents of the matrix.
    pub unsafe fn raw_data(&self) -> *const f32
    {
        &self.data[0]
    }
}

impl PartialEq for Matrix4 {
    fn ne(&self, other: &Matrix4) -> bool {
        for (&ours, &theirs) in self.data.iter().zip(other.data.iter()) {
            if (ours - theirs).abs() > EPSILON {
                return true
            }
        }
        false
    }

    fn eq(&self, other: &Matrix4) -> bool {
        !(self != other)
    }
}
impl Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index<'a>(&'a self, index: (usize, usize)) -> &'a f32 {
        let (row, col) = index;
        assert!(row < 4 && col < 4);
        &self.data[row * 4 + col]
    }
}

impl IndexMut<(usize, usize)> for Matrix4 {
    fn index_mut<'a>(&'a mut self, index: (usize, usize)) -> &'a mut f32 {
        let (row, col) = index;
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

impl Mul<Point> for Matrix4 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self[(0, 0)] * rhs.x + self[(0, 1)] * rhs.y + self[(0, 2)] * rhs.z + self[(0, 3)] * rhs.w,
            y: self[(1, 0)] * rhs.x + self[(1, 1)] * rhs.y + self[(1, 2)] * rhs.z + self[(1, 3)] * rhs.w,
            z: self[(2, 0)] * rhs.x + self[(2, 1)] * rhs.y + self[(2, 2)] * rhs.z + self[(2, 3)] * rhs.w,
            w: self[(3, 0)] * rhs.x + self[(3, 1)] * rhs.y + self[(3, 2)] * rhs.z + self[(3, 3)] * rhs.w,
        }
    }
}

impl Debug for Matrix4 {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(formatter.write_str("\n"));
        for row in 0..4 {
            try!(formatter.write_str("["));
            for col in 0..4 {
                try!(write!(formatter, "{:>+.8}, ", self[(row, col)]));
            }
            try!(formatter.write_str("]\n"));
        }

        Ok(())
    }
}
