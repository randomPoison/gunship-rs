/// Quaternion math used to represent orientation/rotation in 3D space.
///
/// Quaternions are \~\~MATH MAGIC\~\~.
///
/// A quaternion is composed of four components: Three imaginary parts (contained in the vector
/// `v`) and a real part (contained in `w`).
///
/// The product operator uses the [Hamilton product][hamilton product]. For pairwise product
/// use `mul()`.
///
/// TODO: Fill out all the math deets.
///
/// [hamilton product]: https://en.wikipedia.org/wiki/Quaternion#Hamilton_product

use orientation::Orientation;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use vector::Vector3;
use super::{IsZero, Dot};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Quaternion {
    pub v: Vector3,
    pub w: f32,
}

impl Quaternion {
    pub fn new(imaginary: Vector3, real: f32) -> Quaternion {
        Quaternion {
            v: imaginary,
            w: real,
        }
    }

    /// Creates an identity quaternion.
    ///
    /// The identity quaternion is the quaternion that can be multiplied into any other quaternion
    /// without changing it.
    pub fn identity() -> Quaternion {
        Quaternion {
            v: Vector3::zero(),
            w: 1.0,
        }
    }

    /// Gets the length of the quaternion.
    pub fn len(self) -> f32 {
        Quaternion::dot(self, self).sqrt()
    }

    /// Gets the squared length of the quaternion.
    ///
    /// In calculations where you need the squared length of the quaternion, useing `len_sqr()`
    /// allows you to get it without doing a square root operation followed by a square
    /// operation.
    pub fn len_sqr(self) -> f32 {
        Quaternion::dot(self, self)
    }

    /// Normalizes the quaternion to unit length.
    ///
    /// The quaternion must not have a length of zero when calling this method.
    pub fn normalize(&mut self) {
        assert!(!self.is_zero());

        let len = self.len();
        self.v /= len;
        self.w /= len;
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

    /// Calculates the dot product of two quaternions.
    pub fn dot(first: Quaternion, second: Quaternion) -> f32 {
        Vector3::dot(first.v, second.v) + first.w * second.w
    }

    /// Interpolates linearly between two quaternions.
    ///
    /// # Remarks
    ///
    /// This method does not necessarily result in a normalized Quaternion, so the result should
    /// not be used directly to represent a rotation. If you would like to lerp two rotation
    /// quaternions use `Quaternion::nlerp()`.
    pub fn lerp(first: Quaternion, second: Quaternion, t: f32) -> Quaternion {
        first + (second - first) * t
    }

    pub fn inverse(self) -> Quaternion {
        (1.0 / self.len_sqr()) * self.conjugate()
    }

    /// Calculates the quaternion representing the opposite rotation.
    pub fn conjugate(self) -> Quaternion {
        Quaternion {
            v: -self.v,
            w: self.w,
        }
    }
}

impl From<Orientation> for Quaternion {
    fn from(from: Orientation) -> Quaternion {
        from.0
    }
}

impl Default for Quaternion {
    fn default() -> Quaternion {
        Quaternion::identity()
    }
}

impl Add for Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            v: self.v + rhs.v,
            w: self.w + rhs.w,
        }
    }
}

impl AddAssign for Quaternion {
    fn add_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs;
    }
}

impl Sub for Quaternion {
    type Output = Quaternion;

    fn sub(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            v: self.v - rhs.v,
            w: self.w - rhs.w,
        }
    }
}

impl SubAssign for Quaternion {
    fn sub_assign(&mut self, rhs: Quaternion) {
        *self = *self - rhs;
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            v: Vector3::cross(self.v, rhs.v) + rhs.w * self.v + self.w * rhs.v,
            w: self.w * rhs.w - Vector3::dot(self.v, rhs.v),
        }
    }
}

impl MulAssign for Quaternion {
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs; // TODO: No temp object? We'll have to see if LLVM can optimize it away.
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: f32) -> Quaternion {
        Quaternion {
            v: self.v * rhs,
            w: self.w * rhs,
        }
    }
}

impl Mul<Quaternion> for f32 {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        rhs * self
    }
}

impl IsZero for Quaternion {
    fn is_zero(self) -> bool {
        self.len_sqr().is_zero()
    }
}
