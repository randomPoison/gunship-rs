#![feature(raw)]
#![cfg_attr(test, feature(test))]

pub mod point;
pub mod vector;
pub mod matrix;
pub mod color;
pub mod quaternion;

#[cfg(test)]
mod test;

pub use self::point::Point;
pub use self::vector::Vector3;
pub use self::matrix::{Matrix4, Matrix3};
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

pub trait Dot<Other=Self> {
    type Output;

    fn dot(self, rhs: Other) -> Self::Output;
}

impl Dot for [f32; 3] {
    type Output = f32;

    fn dot(self, rhs: [f32; 3]) -> f32 {
        self[0] * rhs[0]
      + self[1] * rhs[1]
      + self[2] * rhs[2]
    }
}

// Doesn't cause ICE.
impl<'a, T> Dot<&'a T> for T where T: Dot<T> + Copy {
    type Output = T::Output;

    fn dot(self, rhs: &T) -> Self::Output {
        self.dot(*rhs)
    }
}

// // Causes ICE.
// impl<'a, T, U> Dot<&'a U> for T where T: Dot<U>, U: Copy {
//     type Output = T::Output;
//
//     fn dot(self, rhs: &U) -> Self::Output {
//         self.dot(*rhs)
//     }
// }
