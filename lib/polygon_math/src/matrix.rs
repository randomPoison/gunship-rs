use std::ops::{Index, IndexMut, Mul};
use std::fmt::{Debug, Formatter, Error};
use std::cmp::PartialEq;

use vector::Vector3;
use orientation::Orientation;
use point::Point;
use quaternion::Quaternion;
use super::{IsZero, Dot};

/// A 4x4 matrix that can be used to represent a combination of translation, rotation, and scale.
///
/// Matrices are row-major.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Matrix4([[f32; 4]; 4]);

impl Matrix4 {
    /// Create a new empy matrix.
    ///
    /// The result matrix is filled entirely with zeroes, it is NOT an identity
    /// matrix. use Matrix4::identity() to get a new identit matrix.
    pub fn new() -> Matrix4 {
        Matrix4([
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0]
        ])
    }

    /// Create a new identity matrix.
    pub fn identity() -> Matrix4 {
        Matrix4([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Create a new translation matrix.
    pub fn translation(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4([
            [1.0, 0.0, 0.0, x  ],
            [0.0, 1.0, 0.0, y  ],
            [0.0, 0.0, 1.0, z  ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Creates a new translation matrix from a point.
    pub fn from_point(point: Point) -> Matrix4 {
        Matrix4::translation(point.x, point.y, point.z)
    }

    /// Creates a new rotation matrix from a set of Euler angles.
    ///
    /// Details
    /// -------
    ///
    /// The resulting matrix will have the rotations applied in the order x -> y -> z.
    pub fn rotation(x: f32, y: f32, z: f32) -> Matrix4 {
        let s1 = x.sin();
        let c1 = x.cos();
        let s2 = y.sin();
        let c2 = y.cos();
        let s3 = z.sin();
        let c3 = z.cos();

        Matrix4([
            [c2 * c3,                -c2 * s3,               s2,       0.0],
            [c1 * s3 + c3 * s1 * s2, c1 * c3 - s1 * s2 * s3, -c2 * s1, 0.0],
            [s1 * s3 - c1 * c3 * s2, c3 * s1 + c1 * s2 * s3, c1 * c2,  0.0],
            [0.0,                    0.0,                    0.0,      1.0],
        ])
    }

    /// Creates a new rotation matrix from a quaternion.
    pub fn from_orientation(q: Orientation) -> Matrix4 {
        q.into()
    }

    pub fn from_matrix3(other: Matrix3) -> Matrix4 {
        Matrix4([
            [other[0][0], other[0][1], other[0][2], 0.0],
            [other[1][0], other[1][1], other[1][2], 0.0],
            [other[2][0], other[2][1], other[2][2], 0.0],
            [0.0,         0.0,         0.0,         1.0],
        ])
    }

    /// Creates a new scale matrix.
    pub fn scale(x: f32, y: f32, z: f32) -> Matrix4 {
        Matrix4([
            [x,   0.0, 0.0, 0.0],
            [0.0, y,   0.0, 0.0],
            [0.0, 0.0, z,   0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_scale_vector(scale: Vector3) -> Matrix4 {
        Matrix4([
            [scale.x, 0.0,     0.0,     0.0],
            [0.0,     scale.y, 0.0,     0.0],
            [0.0,     0.0,     scale.z, 0.0],
            [0.0,     0.0,     0.0,     1.0],
        ])
    }

    pub fn transpose(&self) -> Matrix4 {
        let mut transpose = *self;
        for row in 0..4 {
            for col in (row + 1)..4
            {
                let temp = transpose[row][col];
                transpose[row][col] = transpose[col][row];
                transpose[col][row] = temp;
            }
        }
        transpose
    }

    pub fn x_part(&self) -> Vector3 {
        Vector3::new(self[0][0], self[1][0], self[2][0])
    }

    pub fn y_part(&self) -> Vector3 {
        Vector3::new(self[0][1], self[1][1], self[2][1])
    }

    pub fn z_part(&self) -> Vector3 {
        Vector3::new(self[0][2], self[1][2], self[2][2])
    }

    pub fn translation_part(&self) -> Point {
        Point::new(self[0][3], self[1][3], self[2][3])
    }

    /// Get the matrix data as a raw array.
    pub fn raw_data(&self) -> &[f32; 16] {
        // It's safe to transmute a pointer to data to a &[f32; 16]
        // because the layout in memory is exactly the same.
        unsafe { ::std::mem::transmute(&self.0) }
    }
}

impl From<Matrix3> for Matrix4 {
    fn from(from: Matrix3) -> Matrix4 {
        Matrix4([
            [from[0][0], from[0][1], from[0][2], 0.0],
            [from[1][0], from[1][1], from[1][2], 0.0],
            [from[2][0], from[2][1], from[2][2], 0.0],
            [0.0,        0.0,        0.0,        1.0],
        ])
    }
}

impl From<Orientation> for Matrix4 {
    fn from(q: Orientation) -> Matrix4 {
        let Orientation(Quaternion { v: Vector3 { x, y, z }, w }) = q;

        Matrix4([
            [(w*w + x*x - y*y - z*z), (2.0*x*y - 2.0*w*z),     (2.0*x*z + 2.0*w*y),     0.0],
            [(2.0*x*y + 2.0*w*z),     (w*w - x*x + y*y - z*z), (2.0*y*z - 2.0*w*x),     0.0],
            [(2.0*x*z - 2.0*w*y),     (2.0*y*z + 2.0*w*x),     (w*w - x*x - y*y + z*z), 0.0],
            [0.0,                     0.0,                     0.0,                     1.0],
        ])
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
        &self.0[index]
    }
}

impl IndexMut<usize> for Matrix4 {
    fn index_mut(&mut self, index: usize) -> &mut [f32; 4] {
        debug_assert!(index < 4, "Cannot get matrix row {} in a 4x4 matrix", index);
        &mut self.0[index]
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, other: Matrix4) -> Matrix4 {
        let mut result: Matrix4 = unsafe { ::std::mem::uninitialized() };

        // for row in 0..4 {
        //     for col in 0..4 {
        //         result[row][col] = {
        //             let mut dot_product = 0.0;
        //             for offset in 0..4 {
        //                 dot_product +=
        //                     self[row][offset] *
        //                     other[offset][col];
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

impl Mul<Matrix4> for Point {
    type Output = Point;

    fn mul(self, rhs: Matrix4) -> Point {
        let Point { x, y, z, w } = self;

        Point {
            x: rhs[0][0] * x + rhs[0][1] * y + rhs[0][2] * z + rhs[0][3] * w,
            y: rhs[1][0] * x + rhs[1][1] * y + rhs[1][2] * z + rhs[1][3] * w,
            z: rhs[2][0] * x + rhs[2][1] * y + rhs[2][2] * z + rhs[2][3] * w,
            w: rhs[3][0] * x + rhs[3][1] * y + rhs[3][2] * z + rhs[3][3] * w,
        }
    }
}

impl Mul<Matrix4> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Matrix4) -> Vector3 {
        let Vector3 { x, y, z } = self;

        Vector3 {
            x: rhs[0][0] * x + rhs[0][1] * y + rhs[0][2] * z,
            y: rhs[1][0] * x + rhs[1][1] * y + rhs[1][2] * z,
            z: rhs[2][0] * x + rhs[2][1] * y + rhs[2][2] * z,
        }
    }
}

impl Debug for Matrix4 {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str("\n")?;
        for row in 0..4 {
            formatter.write_str("[")?;
            for col in 0..4 {
                write!(formatter, "{:>+.8}, ", self[row][col])?;
            }
            formatter.write_str("]\n")?;
        }

        Ok(())
    }
}

/// A 3x3 matrix that can be used to represent a combination of rotation and scale.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Matrix3([[f32; 3]; 3]);

impl Matrix3 {
    pub fn identity() -> Matrix3 {
        Matrix3([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    pub fn from_scale_vector(scale: Vector3) -> Matrix3 {
        Matrix3([
            [scale.x, 0.0,     0.0,   ],
            [0.0,     scale.y, 0.0,   ],
            [0.0,     0.0,     scale.z],
        ])
    }

    /// Creates a new rotation matrix from a set of Euler angles.
    ///
    /// Details
    /// -------
    ///
    /// The resulting matrix will have the rotations applied in the order x -> y -> z.
    pub fn rotation(x: f32, y: f32, z: f32) -> Matrix3 {
        let s1 = x.sin();
        let c1 = x.cos();
        let s2 = y.sin();
        let c2 = y.cos();
        let s3 = z.sin();
        let c3 = z.cos();

        Matrix3([
            [c2 * c3,                -c2 * s3,               s2      ],
            [c1 * s3 + c3 * s1 * s2, c1 * c3 - s1 * s2 * s3, -c2 * s1],
            [s1 * s3 - c1 * c3 * s2, c3 * s1 + c1 * s2 * s3, c1 * c2 ],
        ])
    }

    pub fn col(&self, col: usize) -> Vector3 {
        Vector3 {
            x: self[0][col],
            y: self[1][col],
            z: self[2][col],
        }
    }

    pub fn transpose(&self) -> Matrix3 {
        let mut transpose = *self;
        for row in 0..3 {
            for col in (row + 1)..3
            {
                let temp = transpose[row][col];
                transpose[row][col] = transpose[col][row];
                transpose[col][row] = temp;
            }
        }
        transpose
    }

    pub fn as_matrix4(&self) -> Matrix4 {
        Matrix4::from_matrix3(*self)
    }

    pub fn x_part(&self) -> Vector3 {
        Vector3::new(self[0][0], self[1][0], self[2][0])
    }

    pub fn y_part(&self) -> Vector3 {
        Vector3::new(self[0][1], self[1][1], self[2][1])
    }

    pub fn z_part(&self) -> Vector3 {
        Vector3::new(self[0][2], self[1][2], self[2][2])
    }

    /// Get the matrix data as a raw array.
    pub fn raw_data(&self) -> &[f32; 9] {
        // It's safe to transmute a pointer to data to a &[f32; 9]
        // because the layout in memory is exactly the same.
        let Matrix3(ref data) = *self;

        unsafe { &*(data as *const _ as *const [f32; 9]) }
    }
}

impl From<Matrix4> for Matrix3 {
    fn from(from: Matrix4) -> Matrix3 {
        Matrix3([
            [from[0][0], from[0][1], from[0][2]],
            [from[1][0], from[1][1], from[1][2]],
            [from[2][0], from[2][1], from[2][2]],
        ])
    }
}

impl From<Orientation> for Matrix3 {
    fn from(q: Orientation) -> Matrix3 {
        let Quaternion { v: Vector3 { x, y, z }, w } = q.into();

        Matrix3([
            [(w*w + x*x - y*y - z*z), (2.0*x*y - 2.0*w*z),     (2.0*x*z + 2.0*w*y)    ],
            [(2.0*x*y + 2.0*w*z),     (w*w - x*x + y*y - z*z), (2.0*y*z - 2.0*w*x)    ],
            [(2.0*x*z - 2.0*w*y),     (2.0*y*z + 2.0*w*x),     (w*w - x*x - y*y + z*z)],
        ])
    }
}

impl Index<usize> for Matrix3 {
    type Output = [f32; 3];

    fn index(&self, index: usize) -> &[f32; 3] {
        debug_assert!(index < 3, "Cannot get matrix row {} in a 3x3 matrix", index);
        &self.0[index]
    }
}

impl IndexMut<usize> for Matrix3 {
    fn index_mut(&mut self, index: usize) -> &mut [f32; 3] {
        debug_assert!(index < 3, "Cannot get matrix row {} in a 3x3 matrix", index);
        &mut self.0[index]
    }
}

impl Mul for Matrix3 {
    type Output = Matrix3;

    fn mul(self, other: Matrix3) -> Matrix3 {
        let mut result: Matrix3 = unsafe { ::std::mem::uninitialized() };

        for row in 0..3 {
            for col in 0..3 {
                result[row][col] = {
                    self[row].dot(other.col(col))
                };
            }
        }

        result
    }
}

impl Mul<Matrix3> for Point {
    type Output = Point;

    fn mul(self, rhs: Matrix3) -> Point {
        let Point { x, y, z, w } = self;

        Point {
            x: rhs[0][0] * x + rhs[0][1] * y + rhs[0][2] * z,
            y: rhs[1][0] * x + rhs[1][1] * y + rhs[1][2] * z,
            z: rhs[2][0] * x + rhs[2][1] * y + rhs[2][2] * z,
            w: w,
        }
    }
}

impl Mul<Matrix3> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Matrix3) -> Vector3 {
        Vector3 {
            x: rhs[0][0] * self.x + rhs[0][1] * self.y + rhs[0][2] * self.z,
            y: rhs[1][0] * self.x + rhs[1][1] * self.y + rhs[1][2] * self.z,
            z: rhs[2][0] * self.x + rhs[2][1] * self.y + rhs[2][2] * self.z,
        }
    }
}

impl Debug for Matrix3 {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(formatter.write_str("\n"));
        for row in 0..3 {
            try!(formatter.write_str("["));
            for col in 0..3 {
                try!(write!(formatter, "{:>+.8}, ", self[row][col]));
            }
            try!(formatter.write_str("]\n"));
        }

        Ok(())
    }
}
