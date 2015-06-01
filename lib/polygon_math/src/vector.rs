use std::ops::{Mul, Div, Neg};

use super::IsZero;

#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 {
            x: x,
            y: y,
            z: z
        }
    }

    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }

    pub fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn down() -> Vector3 {
        Vector3::new(0.0, -1.0, 0.0)
    }

    pub fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }

    pub fn from_slice(data: &[f32]) -> Vector3 {
        assert!(data.len() == 3);

        Vector3 {
            x: data[0],
            y: data[1],
            z: data[2],
        }
    }

    pub fn cross(first: Vector3, second: Vector3) -> Vector3 {
        Vector3 {
            x: first.y * second.z - first.z * second.y,
            y: first.z * second.x - first.x * second.z,
            z: first.x * second.y - first.y * second.x,
        }
    }

    pub fn normalize(&mut self) {
        let one_over_magnitude = 1.0 / self.magnitude();
        self.x *= one_over_magnitude;
        self.y *= one_over_magnitude;
        self.z *= one_over_magnitude;
    }

    pub fn normalized(&self) -> Vector3 {
        let mut copy = *self;
        copy.normalize();
        copy
    }

    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn dot(&self, rhs: Vector3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    // pub fn cross(&self, rhs: Vector3) -> Vector3 {
    //     Vector3::new(
    //         self.y * rhs.z - self.z * rhs.y,
    //         self.z * rhs.x - self.x * rhs.z,
    //         self.x * rhs.y - self.y * rhs.x)
    // }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vector3> for f32 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        rhs * self
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl Div<Vector3> for f32 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self / rhs.x, self / rhs.y, self / rhs.z)
    }
}

impl IsZero for Vector3 {
    fn is_zero(self) -> bool {
        self.dot(self).is_zero()
    }
}
