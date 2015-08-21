use std::ops::{Index, IndexMut, Mul};
use std::fmt::{Debug, Formatter, Error};
use std::cmp::PartialEq;

use vector::Vector3;
use point::Point;
use quaternion::Quaternion;
use IsZero;

/// A 4x4 matrix that can be used to transform 3D points and vectors.
///
/// Matrices are row-major.
#[repr(C)] #[derive(Clone, Copy)]
pub struct Matrix4 {
    data: [[f32; 4]; 4]
}

impl Matrix4 {
    /// Create a new empy matrix.
    ///
    /// The result matrix is filled entirely with zeroes, it is NOT an identity
    /// matrix. use Matrix4::identity() to get a new identit matrix.
    pub fn new() -> Matrix4 {
        Matrix4 {
            data: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0]
            ]
        }
    }

    /// Create a new identity matrix.
    pub fn identity() -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    /// Create a new translation matrix.
    pub fn translation(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, x  ],
                [0.0, 1.0, 0.0, y  ],
                [0.0, 0.0, 1.0, z  ],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    /// Creates a new translation matrix from a point.
    pub fn from_point(point: Point) -> Matrix4 {
        Matrix4::translation(point.x, point.y, point.z)
    }

    /// Creates a new rotation matrix from a set of Euler angles.
    pub fn rotation(x: f32, y: f32, z: f32) -> Matrix4 {
        let x_rot = Matrix4 {
            data: [
                [1.0, 0.0,      0.0,     0.0],
                [0.0, x.cos(), -x.sin(), 0.0],
                [0.0, x.sin(),  x.cos(), 0.0],
                [0.0, 0.0,      0.0,     1.0],
            ]
        };

        let y_rot = Matrix4 {
            data: [
                [ y.cos(), 0.0, y.sin(), 0.0],
                [ 0.0,     1.0, 0.0,     0.0],
                [-y.sin(), 0.0, y.cos(), 0.0],
                [ 0.0,     0.0, 0.0,     1.0],
            ]
        };

        let z_rot = Matrix4 {
            data: [
                [z.cos(), -z.sin(), 0.0, 0.0],
                [z.sin(),  z.cos(), 0.0, 0.0],
                [0.0,      0.0,     1.0, 0.0],
                [0.0,      0.0,     0.0, 1.0],
            ]
        };

        z_rot * (y_rot * x_rot)
    }

    /// Creates a new rotation matrix from a quaternion.
    pub fn from_quaternion(q: &Quaternion) -> Matrix4 {
        Matrix4 {
            data: [
                [(q.w*q.w + q.x*q.x - q.y*q.y - q.z*q.z), (2.0*q.x*q.y - 2.0*q.w*q.z),             (2.0*q.x*q.z + 2.0*q.w*q.y),             0.0],
                [(2.0*q.x*q.y + 2.0*q.w*q.z),             (q.w*q.w - q.x*q.x + q.y*q.y - q.z*q.z), (2.0*q.y*q.z - 2.0*q.w*q.x),             0.0],
                [(2.0*q.x*q.z - 2.0*q.w*q.y),             (2.0*q.y*q.z + 2.0*q.w*q.x),             (q.w*q.w - q.x*q.x - q.y*q.y + q.z*q.z), 0.0],
                [0.0,                                     0.0,                                     0.0,                                     1.0],
            ]
        }
    }

    /// Creates a new scale matrix.
    pub fn scale(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4 {
            data: [
                [x,   0.0, 0.0, 0.0],
                [0.0, y,   0.0, 0.0],
                [0.0, 0.0, z,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    pub fn from_scale_vector(scale: Vector3) -> Matrix4 {
        Matrix4 {
            data: [
                [scale.x, 0.0,     0.0,     0.0],
                [0.0,     scale.y, 0.0,     0.0],
                [0.0,     0.0,     scale.z, 0.0],
                [0.0,     0.0,     0.0,     1.0],
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
    pub fn raw_data(&self) -> &[f32; 16] {
        // It's safe to transmute a pointer to data to a &[f32; 16]
        // because the layout in memory is exactly the same.
        unsafe { ::std::mem::transmute(&self.data) }
    }
}

impl PartialEq for Matrix4 {
    fn ne(&self, other: &Matrix4) -> bool {
        let our_data = self.raw_data();
        let their_data = other.raw_data();
        for (ours, theirs) in our_data.iter().zip(their_data.iter()) {
            if !(ours - theirs).is_zero() {
                return true;
            }
        }

        false
    }

    fn eq(&self, other: &Matrix4) -> bool {
        !(self != other)
    }
}

impl Index<usize> for Matrix4 {
    type Output = [f32; 4];

    fn index(&self, index: usize) -> &[f32; 4] {
        debug_assert!(index < 4, "Cannot get matrix row {} in a 4x4 matrix", index);
        &self.data[index]
    }
}

impl IndexMut<usize> for Matrix4 {
    fn index_mut(&mut self, index: usize) -> &mut [f32; 4] {
        debug_assert!(index < 4, "Cannot get matrix row {} in a 4x4 matrix", index);
        &mut self.data[index]
    }
}

impl Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &f32 {
        let (row, col) = index;
        assert!(row < 4 && col < 4);
        &self.data[row][col]
    }
}

impl IndexMut<(usize, usize)> for Matrix4 {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut f32 {
        let (row, col) = index;
        assert!(row < 4 && col < 4);
        &mut self.data[row][col]
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, other: Matrix4) -> Matrix4 {
        let mut result: Matrix4 = unsafe { ::std::mem::uninitialized() };

        // for row in 0..4 {
        //     for col in 0..4 {
        //         result[(row, col)] = {
        //             let mut dot_product = 0.0;
        //             for offset in 0..4 {
        //                 dot_product +=
        //                     self[(row, offset)] *
        //                     other[(offset, col)];
        //             }
        //             dot_product
        //         };
        //     }
        // }

        result[0][0] = (self[0][0] * other[0][0]) + (self[0][1] * other[1][0]) + (self[0][2] * other[2][0]) + (self[0][3] * other[3][0]);
        result[0][1] = (self[0][0] * other[0][1]) + (self[0][1] * other[1][1]) + (self[0][2] * other[2][1]) + (self[0][3] * other[3][1]);
        result[0][2] = (self[0][0] * other[0][2]) + (self[0][1] * other[1][2]) + (self[0][2] * other[2][2]) + (self[0][3] * other[3][2]);
        result[0][3] = (self[0][0] * other[0][3]) + (self[0][1] * other[1][3]) + (self[0][2] * other[2][3]) + (self[0][3] * other[3][3]);
        result[1][0] = (self[1][0] * other[0][0]) + (self[1][1] * other[1][0]) + (self[1][2] * other[2][0]) + (self[1][3] * other[3][0]);
        result[1][1] = (self[1][0] * other[0][1]) + (self[1][1] * other[1][1]) + (self[1][2] * other[2][1]) + (self[1][3] * other[3][1]);
        result[1][2] = (self[1][0] * other[0][2]) + (self[1][1] * other[1][2]) + (self[1][2] * other[2][2]) + (self[1][3] * other[3][2]);
        result[1][3] = (self[1][0] * other[0][3]) + (self[1][1] * other[1][3]) + (self[1][2] * other[2][3]) + (self[1][3] * other[3][3]);
        result[2][0] = (self[2][0] * other[0][0]) + (self[2][1] * other[1][0]) + (self[2][2] * other[2][0]) + (self[2][3] * other[3][0]);
        result[2][1] = (self[2][0] * other[0][1]) + (self[2][1] * other[1][1]) + (self[2][2] * other[2][1]) + (self[2][3] * other[3][1]);
        result[2][2] = (self[2][0] * other[0][2]) + (self[2][1] * other[1][2]) + (self[2][2] * other[2][2]) + (self[2][3] * other[3][2]);
        result[2][3] = (self[2][0] * other[0][3]) + (self[2][1] * other[1][3]) + (self[2][2] * other[2][3]) + (self[2][3] * other[3][3]);
        result[3][0] = (self[3][0] * other[0][0]) + (self[3][1] * other[1][0]) + (self[3][2] * other[2][0]) + (self[3][3] * other[3][0]);
        result[3][1] = (self[3][0] * other[0][1]) + (self[3][1] * other[1][1]) + (self[3][2] * other[2][1]) + (self[3][3] * other[3][1]);
        result[3][2] = (self[3][0] * other[0][2]) + (self[3][1] * other[1][2]) + (self[3][2] * other[2][2]) + (self[3][3] * other[3][2]);
        result[3][3] = (self[3][0] * other[0][3]) + (self[3][1] * other[1][3]) + (self[3][2] * other[2][3]) + (self[3][3] * other[3][3]);

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
