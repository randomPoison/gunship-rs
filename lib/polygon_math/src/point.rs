use std::ops::{Sub, Add, AddAssign, Neg};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::mem;
use std::f32;
use std::slice;

use vector::Vector3;

/// A point in 3D space.
///
/// Points are represented as cartesian coordinates with an `x`, `y`, and `z` position, as well as
/// a `w` homogeneous coordinate for the purposes of linear algebra calculations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Point {
    /// Creates a new point with the given coordinates.
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point {
            x: x,
            y: y,
            z: z,
            w: 1.0,
        }
    }

    /// Creates a new point at the origin `(0.0, 0.0, 0.0, 1.0)`.
    pub fn origin() -> Point {
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    /// Creates a new point at the minimum representable coordinate.
    ///
    /// The minimum point is the one which has `f32::MIN` for its x, y, and z coordinates, and 1.0
    /// for its w coordinate.
    pub fn min() -> Point {
        Point::new(f32::MIN, f32::MIN, f32::MIN)
    }

    /// Creates a new point at the maximum representable coordinate.
    ///
    /// The maximum point is the one which has `f32::MAX` for its x, y, and z coordinates, and 1.0
    /// for its w coordinate.
    pub fn max() -> Point {
        Point::new(f32::MAX, f32::MAX, f32::MAX)
    }

    /// Calculates the distance in (in abstract units) between two points.
    ///
    /// This method uses `sqrt()` to calculate the distance between the points. If the exact
    /// distance isn't necessary (e.g. when comparing two or more distances to each other) use
    /// `distance_sqr()` as it is cheaper to calculate.
    pub fn distance(&self, other: &Point) -> f32 {
        self.distance_sqr(other).sqrt()
    }

    /// Calculates the squared distance between two points.
    ///
    /// This method is offered as an optimization over `distance()` because there are some cases
    /// where the square distance is sufficent and calculating the squared distance avoids a
    /// relatively costly square root calculation.
    pub fn distance_sqr(&self, other: &Point) -> f32 {
        let diff_x = self.x - other.x;
        let diff_y = self.y - other.y;
        let diff_z = self.z - other.z;

        diff_x * diff_x + diff_y * diff_y + diff_z * diff_z
    }

    pub fn as_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    pub fn as_array(&self) -> &[f32; 4] {
        unsafe { mem::transmute(self) }
    }

    pub fn as_slice_of_arrays(points: &[Point]) -> &[[f32; 4]] {
        let ptr = points.as_ptr() as *const _;
        unsafe { slice::from_raw_parts(ptr, points.len()) }
    }

    pub fn as_ref(points: &[Point]) -> &[f32] {
        let ptr = points.as_ptr() as *const _;
        let len = points.len() * 4;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    pub fn slice_from_f32_slice(raw: &[f32]) -> &[Point] {
        assert!(
            raw.len() % 4 == 0,
            "To convert a slice of f32 to a slice of Point it must have a length that is a \
             multiple of 4");

        unsafe { slice::from_raw_parts(raw.as_ptr() as *const Point, raw.len() / 4) }
    }
}

impl Sub for Point {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl AddAssign<Vector3> for Point {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w = 1.0;
    }
}

impl Add<Vector3> for Point {
    type Output = Point;

    fn add(mut self, rhs: Vector3) -> Point {
        self += rhs;
        self
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

impl PartialOrd for Point {
    /// Ordering for points is defined by ordering the ordering precedence as x > y > z.
    ///
    /// TODO: Elaborate on ordering for points in vectors in the module documentation?
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        debug_assert!(self.w == 1.0 && other.w == 1.0,
                      "Points must be normalized before comparison");

        if self.x < other.x {
            Some(Ordering::Less)
        } else if self.x > other.x {
            Some(Ordering::Greater)
        } else if self.y < other.y {
            Some(Ordering::Less)
        } else if self.y > other.y {
            Some(Ordering::Greater)
        } else if self.z < other.z {
            Some(Ordering::Less)
        } else if self.z > other.z {
            Some(Ordering::Greater)
        } else if self.x == other.x && self.y == other.y && self.z == other.z {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl Ord for Point {
    /// Super bad-nasty implementation of Ord for Point.
    ///
    /// This is so that we can use cmp::min() and cmp::max() with Point, but we have to settle
    /// for panicking when a strict ordering can't be determined. We could also choose to define
    /// an arbitrary ordering for NaN elements, but if a point has NaN coordinates something has
    /// likely gone wrong so panicking will help even stranger bugs from appearing.
    fn cmp(&self, other: &Point) -> Ordering {
        match PartialOrd::partial_cmp(self, other) {
            Some(ordering) => ordering,
            None => {
                panic!(
                    "Trying to compare points {:?} and {:?} when one as NaN coordinates",
                    self,
                    other)
            }
        }
    }
}

impl From<(f32, f32, f32)> for Point {
    fn from(from: (f32, f32, f32)) -> Point {
        Point {
            x: from.0,
            y: from.1,
            z: from.2,
            w: 1.0,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Point {
    fn from(from: (f32, f32, f32, f32)) -> Point {
        Point {
            x: from.0,
            y: from.1,
            z: from.2,
            w: from.3,
        }
    }
}

impl<'a> From<&'a [f32]> for Point {
    fn from(from: &[f32]) -> Point {
        assert!(from.len() == 3 || from.len() == 4);

        Point {
            x: from[0],
            y: from[1],
            z: from[2],
            w: if from.len() == 4 { from[3] } else { 1.0 },
        }
    }
}

impl From<Vector3> for Point {
    /// Creates a new `Point` from a `Vector3`.
    ///
    /// This behaves as if the `Vector3` had been added to the origin, resulting in a `Point` with
    /// the same x, y, and z coordinates as the original `Vector3` had. The conversion can be
    /// expressed as `<x, y, z> => (x, y, z, 1.0)`.
    fn from(from: Vector3) -> Point {
        Point {
            x: from.x,
            y: from.y,
            z: from.z,
            w: 1.0,
        }
    }
}
