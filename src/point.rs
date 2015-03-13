pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

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
