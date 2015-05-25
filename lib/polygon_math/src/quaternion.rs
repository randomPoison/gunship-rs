use std::ops::Mul;

use vector::Vector3;
use matrix::Matrix4;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    /// Creates an identity quaternion.
    ///
    /// The identity quaternion is the quaternion that can be multiplied into any other quaternion
    /// without changing it.
    pub fn identity() -> Quaternion {
        Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a quaternion from an axis and a rotation around that axis.
    ///
    /// # Params
    ///
    /// - axis - The axis being used to represent the rotation. This should
    ///   be normalized before being passed into `axis_angle()`.
    pub fn axis_angle(axis: Vector3, angle: f32) -> Quaternion {
        Quaternion {
            w: (angle * 0.5).cos(),
            x: (angle * 0.5).sin() * axis.x,
            y: (angle * 0.5).sin() * axis.y,
            z: (angle * 0.5).sin() * axis.z,
        }
    }

    /// Creates a quaternion from a set of euler angles.
    pub fn from_eulers(x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion::axis_angle(Vector3(1.0, 0.0, 0.0), x)
      * Quaternion::axis_angle(Vector3(0.0, 1.0, 0.0), y)
      * Quaternion::axis_angle(Vector3(0.0, 0.0, 1.0), z);
    }

    /// Converts the quaternion to the corresponding rotation matrix.
    pub fn as_matrix(&self) -> Matrix4 {
        Matrix4::from_quaternion(self)
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: (self.w * rhs.w) - (self.x * rhs.x) - (self.y * rhs.y) - (self.z * rhs.z),
            x: (self.w * rhs.x) + (self.x * rhs.w) + (self.y * rhs.z) - (self.z * rhs.y),
            y: (self.w * rhs.y) - (self.x * rhs.z) + (self.y * rhs.w) + (self.z * rhs.x),
            z: (self.w * rhs.z) + (self.x * rhs.y) - (self.y * rhs.x) + (self.z * rhs.w),
        }
    }
}
