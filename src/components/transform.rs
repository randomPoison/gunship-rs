use math::vector::{vector3, Vector3};
use entity::Entity;

pub struct TransformManager {
    transforms: Vec<Transform>
}

pub struct Transform {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3
}

impl TransformManager {
    pub fn new() -> TransformManager {
        TransformManager {
            transforms: Vec::new()
        }
    }

    pub fn create(&mut self, entity: Entity) -> &Transform {
        let index = self.transforms.len();
        self.transforms.push(Transform::new());
        &self.transforms[index]
    }
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale:    Vector3::one()
        }
    }
}
