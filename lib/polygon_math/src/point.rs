use std::ops::{Sub, Add, Neg};

use vector::Vector3;

/// A point in 3D space.
///
/// Points are represented as cartesian coordinates
/// with an `x`, `y`, and `z` position, as well as
/// a `w` homogeneous coordinate for the purposes
/// of linear algebra calculations.
#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point {
            x: x,
            y: y,
            z: z,
            w: 1.0
        }
    }

    pub fn from_slice(data: &[f32]) -> Point {
        assert!(data.len() == 3 || data.len() == 4);

        Point {
            x: data[0],
            y: data[1],
            z: data[2],
            w: if data.len() == 4 {
                data[4]
            } else {
                1.0
            }
        }
    }

    pub fn origin() -> Point {
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    pub fn as_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    pub unsafe fn raw_data(&self) -> *const f32 {
        &self.x
    }
}

impl Sub for Point {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Add<Vector3> for Point {
    type Output = Point;

    fn add(self, rhs: Vector3) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: 1.0,
        }
    }
}

impl Sub<Vector3> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector3) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: 1.0,
        }
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: 1.0,
        }
    }
}
