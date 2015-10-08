use std::ops::{Sub, Add, Neg};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::mem;
use std::f32;
use std::raw::Slice;
use std::slice;

use vector::Vector3;

/// A point in 3D space.
///
/// Points are represented as cartesian coordinates
/// with an `x`, `y`, and `z` position, as well as
/// a `w` homogeneous coordinate for the purposes
/// of linear algebra calculations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

    /// TODO: Implement the `From` trait rather than making a standalone method.
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

    pub fn min() -> Point {
        Point::new(f32::MIN, f32::MIN, f32::MIN)
    }

    pub fn max() -> Point {
        Point::new(f32::MAX, f32::MAX, f32::MAX)
    }

    pub fn as_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    pub fn as_array(&self) -> &[f32; 4] {
        unsafe { mem::transmute(self) }
    }

    pub fn as_ref(points: &[Point]) -> &[f32] {
        unsafe {
            let Slice { data, len } = mem::transmute(points);
            slice::from_raw_parts(data, len * 4)
        }
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

/// We lie about Point being Eq because it's needed for Ord. For our purposes we don't
/// care that it's not technically true according to the spec.
impl Eq for Point {}

impl Ord for Point {
    /// Super bad-nasty implementation of Ord for Point.
    ///
    /// This is so that we can use cmp::min() and cmp::max() with Point, but we have to settle
    /// for panicking when a strict ordering can't be determined. We could also choose to define
    /// an arbitrary ordering for NaN elements, but if a point has NaN coordinates something has
    /// likely gone wrong so panicking will help even stranger bugs from appearing.
    fn cmp(&self, other: &Point) -> Ordering {
        PartialOrd::partial_cmp(self, other)
        .expect(&*format!(
            "Trying to compare points {:?} and {:?} where one as NaN coordinates",
            self,
            other))
    }
}
