#[macro_use]
pub mod point;
pub mod vector;
#[macro_use]
pub mod matrix;
pub mod color;

#[cfg(test)]
mod test;

pub use self::point::Point;
pub use self::vector::Vector3;
pub use self::matrix::Matrix4;
pub use self::color::Color;
