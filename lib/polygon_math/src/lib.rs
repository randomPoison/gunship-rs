#![feature(const_fn)]
#![feature(slice_patterns)]
#![cfg_attr(test, feature(test))]

pub mod color;
pub mod matrix;
pub mod orientation;
pub mod point;
pub mod quaternion;
pub mod vector;

#[cfg(test)]
mod test;

pub use color::Color;
pub use matrix::{Matrix3, Matrix4};
pub use orientation::Orientation;
pub use point::Point;
pub use std::f32::consts::PI;
pub use vector::{Vector2, Vector3};

use std::ops::{Rem, Add};

pub const EPSILON: f32 = 1e-6;
pub const TAU: f32 = 2.0 * PI;

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

pub trait Lerp {
    fn lerp(t: f32, from: Self, to: Self) -> Self;
}

impl Lerp for f32 {
    fn lerp(t: f32, from: f32, to: f32) -> f32 {
        from + (to - from) * t
    }
}

pub trait Modulo<Rhs = Self> {
    type Output;

    fn modulo(self, divisor: Rhs) -> Self::Output;
}

impl<T> Modulo for T
    where
    T: Copy,
    T: Rem<Output=T>,
    T: Add<Output=T>,
{
    type Output = T;

    fn modulo(self, divisor: T) -> T {
        ((self % divisor) + divisor) % divisor
    }
}
