use std::ops::{Sub};

use vector::{vector3, Vector3};

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
    pub w: f32
}

impl Point {
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
}

impl Sub for Point {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Vector3 {
        vector3(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// Utility macro for quickly defining hard-coded points
///
/// # Examples
///
/// ```rust
/// # #[macro_use] extern crate "render_math" as math;
/// # use math::point::Point;
/// # fn main() {
///
/// let point = point!(0.0, 0.0, 0.0);
///
/// // equivalent to:
/// let point = Point {
///     x: 0.0,
///     y: 0.0,
///     z: 0.0,
///     w: 1.0
/// };
///
/// # }
/// ```
#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr, $z:expr) => {
        Point {
            x: $x,
            y: $y,
            z: $z,
            w: 1.0
        }
    };
}
