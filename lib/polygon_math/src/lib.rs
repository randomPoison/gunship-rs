pub mod point;
pub mod vector;
pub mod matrix;
pub mod color;
pub mod quaternion;

#[cfg(test)]
mod test;

pub use self::point::Point;
pub use self::vector::Vector3;
pub use self::matrix::Matrix4;
pub use self::color::Color;
pub use self::quaternion::Quaternion;

pub const EPSILON: f32 = 1e-6;

pub trait IsZero {
    fn is_zero(self) -> bool;
}

impl IsZero for f32 {
    fn is_zero(self) -> bool {
        self.abs() < EPSILON
    }
}

pub trait Clamp {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Clamp for f32 {
    fn clamp(self, min: f32, max: f32) -> f32 {
        f32::min(f32::max(self, min), max)
    }
}
