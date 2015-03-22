/// A point in 3D space.
///
/// Points are represented as cartesian coordinates
/// with an `x`, `y`, and `z` position, as well as
/// a `w` homogeneous coordinate for the purposes
/// of linear algebra calculations.
#[repr(C)] #[derive(Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
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
