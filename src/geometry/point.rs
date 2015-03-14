pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

/// Utility macro for quickly defining hard-coded points.
///
/// # Examples
///
/// ```
/// let point = point!(0.0, 0.0, 0.0);
///
/// // equivalent to:
/// let point = Point {
///     x: 0.0,
///     y: 0.0,
///     z: 0.0,
///     w: 1.0
/// };
/// ```
#[macro_export]
macro_rules! point {
    ( $x:expr, $y:expr, $z:expr ) => {
        Point {
            x: $x,
            y: $y,
            z: $z,
            w: 1.0
        }
    };
}
