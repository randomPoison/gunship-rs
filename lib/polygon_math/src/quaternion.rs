use std::ops::Mul;
use std::f32::consts::PI;

use vector::Vector3;
use matrix::*;
use super::{IsZero, Clamp};

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
        assert!(axis.is_normalized());

        let s = (angle * 0.5).sin();
        Quaternion {
            w: (angle * 0.5).cos(),
            x: s * axis.x,
            y: s * axis.y,
            z: s * axis.z,
        }.normalized()
    }

    /// Creates a quaternion that rotates an object to look in the specified direction.
    pub fn look_rotation(forward: Vector3, up: Vector3) -> Quaternion {
        assert!(!forward.is_zero());
        assert!(!up.is_zero());

        let source = Vector3::forward();
        let forward = forward.normalized();
        let up = up.normalized();

        let dot = source.dot(forward);

        if (dot + 1.0).is_zero() {
            // vector a and b point exactly in the opposite direction,
            // so it is a 180 degrees turn around the up-axis
            return Quaternion::axis_angle(up, PI)
        }

        if (dot - 1.0).is_zero() {
            // Vector a and b point exactly in the same direction
            // so we return the identity quaternion.
            return Quaternion::identity()
        }

        let rot_angle = dot.acos();
        let rot_axis = Vector3::cross(source, forward).normalized();// source.cross(forward).normalized();
        return Quaternion::axis_angle(rot_axis, rot_angle)

        // TODO: Correctly take the up vector into account.
    }

    /// Creates a quaternion from a set of euler angles.
    pub fn from_eulers(x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), x)
      * Quaternion::axis_angle(Vector3::new(0.0, 1.0, 0.0), y)
      * Quaternion::axis_angle(Vector3::new(0.0, 0.0, 1.0), z)
    }

    /// Converts the quaternion to the corresponding rotation matrix.
    pub fn as_matrix4(self) -> Matrix4 {
        Matrix4::from_quaternion(self)
    }

    pub fn as_matrix3(self) -> Matrix3 {
        Matrix3::from_quaternion(self)
    }

    /// Retrieves the rotation represented by the quaternion as a rotation about an axis.
    ///
    /// The returned axis will always be normalized.
    pub fn as_axis_angle(&self) -> (Vector3, f32) {
        assert!(self.is_normalized());

        let angle = 2.0 * self.w.acos();
        let s = (1.0 - self.w * self.w).sqrt();
        if s.is_zero() {
            // If s is 0, axis is arbitrary.
            (Vector3::new(1.0, 0.0, 0.0), angle)
        } else {
            (Vector3::new(self.x / s, self.y / s, self.z / s).normalized(), angle)
        }
    }

    /// Retrieves the rotation represented by the quaternion as euler angles.
    pub fn as_eulers(&self) -> Vector3 {
        assert!(self.is_normalized());

        let x = f32::atan2(
            2.0 * (self.w * self.x + self.y * self.z),
            1.0 - 2.0 * (self.x * self.x + self.y * self.y));
        let y = (2.0 * (self.w * self.y - self.z * self.x)).asin();
        let z = f32::atan2(
            2.0 * (self.w * self.z + self.x * self.y),
            1.0 - 2.0 * (self.y * self.y + self.z * self.z));

        Vector3::new(x, y, z)
    }

    /// Normalizes the quaternion to unit length.
    ///
    /// The quaternion must not have a length of zero when calling this method.
    pub fn normalize(&mut self) {
        assert!(!self.is_zero());

        let length = (self.w * self.w
                    + self.x * self.x
                    + self.y * self.y
                    + self.z * self.z).sqrt();

        self.w /= length;
        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    /// Creates a normalized copy of the quaternion.
    ///
    /// The quaternion must not have a length of zero when calling this method.
    pub fn normalized(&self) -> Quaternion {
        let mut temp = *self;
        temp.normalize();
        temp
    }

    /// Determines if the quaternion is normalized (has a length of 1.0).
    pub fn is_normalized(&self) -> bool {
        let mag_sqrd = Quaternion::dot(*self, *self);
        (mag_sqrd - 1.0).is_zero()
    }

    pub fn repeat(&self, repeat: f32) -> Quaternion {
        let (axis, angle) = self.as_axis_angle();
        Quaternion::axis_angle(axis, angle * repeat)
    }

    /// Calculates the dot product of two quaternions.
    pub fn dot(first: Quaternion, second: Quaternion) -> f32 {
        (first.w * second.w
       + first.x * second.x
       + first.y * second.y
       + first.z * second.z)
    }

    /// Interpolates linearly between two quaternions.
    ///
    /// # Remarks
    ///
    /// This method does not necessarily result in a normalized Quaternion, so the result should
    /// not be used directly to represent a rotation. If you would like to lerp two rotation
    /// quaternions use `Quaternion::nlerp()`.
    pub fn lerp(first: Quaternion, second: Quaternion, t: f32) -> Quaternion {
        // first + t * (second - first)
        second.sub(first).mul(t).add(first)
    }

    /// Interpolates linearly between two quaternions using a normalized lerp.
    pub fn nlerp(first: Quaternion, second: Quaternion, t: f32) -> Quaternion {
        Quaternion::lerp(first, second, t).normalized()
    }

    /// Calculates the spherical linear interpolation between two quaternions.
    ///
    /// # Remarks
    ///
    /// While slerp is generally the go-to method for interpolating quaternions, it's computationaly
    /// expensive and has some undesirable properties. `Quaternion::nlerp()` is often more appropriate unless it's
    /// absolutely necessary that the interpolation have a constant velocity. For a better discussion
    /// of the different methods of interpolating quaternions see [this article by Jonathan Blow]
    /// (http://number-none.com/product/Understanding%20Slerp,%20Then%20Not%20Using%20It/).
    pub fn slerp(first: Quaternion, second: Quaternion, t: f32) -> Quaternion {
        assert!(first.is_normalized());
        assert!(second.is_normalized());

        // Compute the cosine of the angle between the two vectors.
        let dot = Quaternion::dot(first, second);

        const DOT_THRESHOLD: f32 = 0.9995;
        if dot > DOT_THRESHOLD {
            // If the inputs are too close for comfort, linearly interpolate
            // and normalize the result.
            return Quaternion::nlerp(first, second, t);
        }

        dot.clamp(-1.0, 1.0);     // Robustness: Stay within domain of acos()
        let theta_0 = dot.acos(); // theta_0 = angle between input vectors
        let theta = theta_0 * t;  // theta = angle between first and result

        let normal = (second.sub(first).mul(dot)).normalized(); // { first, normal } is now an orthonormal basis

        // TODO: We shouldn't need to normalize here since both inputs are normalized,
        //       the result was introducing error. Figure out if there's something else we
        //       can do to improve accuracy.
        //      first *   theta.cos()  +   normal *   theta.sin()
        return (first.mul(theta.cos()).add(normal.mul(theta.sin()))).normalized();
    }

    fn mul(self, rhs: f32) -> Quaternion {
        Quaternion {
            w: self.w * rhs,
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }

    fn sub(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w - rhs.w,
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }

    fn add(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w + rhs.w,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        assert!(self.is_normalized());
        assert!(rhs.is_normalized());

        Quaternion {
            w: (self.w * rhs.w) - (self.x * rhs.x) - (self.y * rhs.y) - (self.z * rhs.z),
            x: (self.w * rhs.x) + (self.x * rhs.w) + (self.y * rhs.z) - (self.z * rhs.y),
            y: (self.w * rhs.y) - (self.x * rhs.z) + (self.y * rhs.w) + (self.z * rhs.x),
            z: (self.w * rhs.z) + (self.x * rhs.y) - (self.y * rhs.x) + (self.z * rhs.w),
        }.normalized()
    }
}

// TODO: impl Mul<Vector3> for Quaternion (or maybe other way around).

impl IsZero for Quaternion {
    fn is_zero(self) -> bool {
        (self.w * self.w
       + self.x * self.x
       + self.y * self.y
       + self.z * self.z).is_zero()
    }
}
