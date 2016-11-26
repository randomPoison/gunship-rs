use quaternion::Quaternion;
use std::ops::{Add, AddAssign, Sub, SubAssign, Div, DivAssign, Mul, MulAssign};
use super::{IsZero, Dot, PI};
use vector::Vector3;

/// An orientation in 3D space.
///
/// An instance of `Orientation` can represent any combination of rotations in 3D space. The type
/// provides ways to manipulate 3D orientations, and provides a higher-level way of talking
/// about orientation than using unit quaternions directly.
///
/// TODO: Expand on the mathy stuff:
/// - "Quaternion" really means "unit quaternion", since that's the only case we care about.
/// - Multiplication means composing two rotations.
/// - "Grassman product" is technically the type of multiplication used.
/// - Multiplying by a `Vector3` rotates the vector.
/// - Concatenation happens left-to-right, e.g. `v * q1 * q2 * q3` rotates `v` by `q1`, then `q2`,
///   then `q3`. In theory that order shouldn't really matter, right? Rotation is commutative?
// TODO: The innner quaternion should be `pub(crate)` once pub_restricted is stabilized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orientation(pub Quaternion);

impl Orientation {
    /// Creates a new orientation representing no rotation.
    pub fn new() -> Orientation {
        Orientation(Quaternion::identity())
    }

    /// Creates an orientation from an axis and a rotation around that axis.
    ///
    /// # Params
    ///
    /// - axis - The axis being used to represent the rotation. This should
    ///   be normalized before being passed into `axis_angle()`.
    pub fn axis_angle(axis: Vector3, angle: f32) -> Orientation {
        assert!(axis.is_normalized());

        // TODO: Do we *need* to normalize the result? Shouldn't it already be normalized?
        let half_angle = angle * 0.5;
        let q = Quaternion::new(axis * half_angle.sin(), half_angle.cos()).normalized();
        Orientation(q)
    }

    /// Creates an orientation that rotates an object to look in the specified direction.
    pub fn look_rotation(forward: Vector3, up: Vector3) -> Orientation {
        assert!(!forward.is_zero());
        assert!(!up.is_zero());

        let source = Vector3::forward();
        let forward = forward.normalized();
        let up = up.normalized();

        let dot = source.dot(forward);

        if (dot + 1.0).is_zero() {
            // vector a and b point exactly in the opposite direction,
            // so it is a 180 degrees turn around the up-axis
            return Orientation::axis_angle(up, PI)
        }

        if (dot - 1.0).is_zero() {
            // Vector a and b point exactly in the same direction
            // so we return the identity quaternion.
            return Orientation::new()
        }

        // let rot_angle = dot.acos();
        // let rot_axis = Vector3::cross(source, forward).normalized();// source.cross(forward).normalized();
        // return Orientation::axis_angle(rot_axis, rot_angle)

        // TODO: Correctly take the up vector into account.
        unimplemented!();
    }

    /// Creates a quaternion from a set of euler angles.
    pub fn from_eulers(x: f32, y: f32, z: f32) -> Orientation {
        Orientation::axis_angle(Vector3::new(1.0, 0.0, 0.0), x)
      + Orientation::axis_angle(Vector3::new(0.0, 1.0, 0.0), y)
      + Orientation::axis_angle(Vector3::new(0.0, 0.0, 1.0), z)
    }

    /// Retrieves the rotation represented by the `Orientation` as a rotation about an axis.
    ///
    /// The returned axis will always be normalized.
    pub fn as_axis_angle(self) -> (Vector3, f32) {
        assert!(self.0.is_normalized());

        let angle = 2.0 * self.0.w.acos();
        let s = (1.0 - self.0.w * self.0.w).sqrt();
        if s.is_zero() {
            // If s is 0, axis is arbitrary.
            (Vector3::new(1.0, 0.0, 0.0), angle)
        } else {
            ((self.0.v /  s).normalized(), angle)
        }
    }

    /// Retrieves the rotation represented by the quaternion as euler angles.
    pub fn as_eulers(mut self) -> Vector3 {
        self.0.normalize();

        // Extract quaternion components.
        let Vector3 { x, y, z } = self.0.v;
        let w = self.0.w;

        // Implementation taken from here: http://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToEuler/
        let test = x * y + z * w;

        // Test for singularity at north pole.
        if test > 0.499 {
            let heading = 2.0 * f32::atan2(x,w);
            let attitude = PI / 2.0;
            let bank = 0.0;
            return Vector3::new(bank, heading, attitude);
        }

        // Test for singularity at south pole.
        if test < -0.499 {
            let heading = -2.0 * f32::atan2(x,w);
            let attitude = -PI / 2.0;
            let bank = 0.0;
            return Vector3::new(bank, heading, attitude);
        }

        let sqx = x * x;
        let sqy = y * y;
        let sqz = z * z;
        let heading = f32::atan2(2.0 * y * w - 2.0 * x * z, 1.0 - 2.0 * sqy - 2.0 * sqz);
        let attitude = f32::asin(2.0 * test);
        let bank = f32::atan2(2.0 * x * w - 2.0 * y * z, 1.0 - 2.0 * sqx - 2.0 * sqz);
        return Vector3::new(bank, heading, attitude);
    }

    /// Gets the right direction for the orientation.
    ///
    /// The right direction for the orientation is the global right vector (positive x axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn right(self) -> Vector3 {
        self * Vector3::right()
    }

    /// Gets the left direction for the orientation.
    ///
    /// The left for the orientation is the global right vector (negative x axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn left(self) -> Vector3 {
        self * Vector3::left()
    }

    /// Gets the up direction for the orientation.
    ///
    /// The up direction for the orientation is the global up vector (positive y axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn up(self) -> Vector3 {
        self * Vector3::up()
    }

    /// Gets the down direction for the orientation.
    ///
    /// The down direction for the orientation is the global down vector (negative y axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn down(self) -> Vector3 {
        self * Vector3::down()
    }

    /// Gets the forward direction for the orientation.
    ///
    /// The forward direction for the orientation is the global forward vector (negative z axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn forward(self) -> Vector3 {
        self * Vector3::forward()
    }

    /// Gets the back direction for the orientation.
    ///
    /// The back direction for the orientation is the global back vector (positive z axis) as
    /// rotated by the orientation. The returned vector will be normalized.
    pub fn back(self) -> Vector3 {
        self * Vector3::back()
    }
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation::new()
    }
}

impl Add for Orientation {
    type Output = Orientation;

    fn add(self, rhs: Orientation) -> Orientation {
        Orientation(self.0 * rhs.0)
    }
}

impl AddAssign for Orientation {
    fn add_assign(&mut self, rhs: Orientation) {
        self.0 *= rhs.0
    }
}

impl Sub for Orientation {
    type Output = Orientation;

    fn sub(self, rhs: Orientation) -> Orientation {
        Orientation(rhs.0.conjugate() * self.0)
    }
}

impl SubAssign for Orientation {
    fn sub_assign(&mut self, rhs: Orientation) {
        self.0 = rhs.0.conjugate() * self.0;
    }
}

impl Mul<f32> for Orientation {
    type Output = Orientation;

    fn mul(self, rhs: f32) -> Orientation {
        let (axis, angle) = self.as_axis_angle();
        Orientation::axis_angle(axis, angle * rhs)
    }
}

impl MulAssign<f32> for Orientation {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<Vector3> for Orientation {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        let rhs_quat = Quaternion { v: rhs, w: 1.0 };
        let result_quat = self.0 * rhs_quat * self.0.conjugate();

        // TODO: Should `result_quat.w` always be 1.0? If so should we be asserting or something?
        // Should we be normalizing the result quaterning before extracting the vector?
        result_quat.v
    }
}

impl Div<f32> for Orientation {
    type Output = Orientation;

    fn div(self, rhs: f32) -> Orientation {
        let (axis, angle) = self.as_axis_angle();
        Orientation::axis_angle(axis, angle / rhs)
    }
}

impl DivAssign<f32> for Orientation {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
