use std::collections::HashMap;

use math::vector::Vector3;
use math::matrix::Matrix4;
use math::point::Point;
use ecs::Entity;

pub struct TransformManager {
    transforms: Vec<Transform>,
    indices: HashMap<Entity, usize>,
}

impl TransformManager {
    pub fn new() -> TransformManager {
        TransformManager {
            transforms: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn create(&mut self, entity: Entity) -> &mut Transform {
        let index = self.transforms.len();
        self.transforms.push(Transform::new());
        self.indices.insert(entity, index);
        &mut self.transforms[index]
    }

    pub fn get(&self, entity: Entity) -> &Transform {
        let index = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        &self.transforms[index]
    }

    pub fn get_mut(&mut self, entity: Entity) -> &mut Transform {
        let index = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        &mut self.transforms[index]
    }
}

#[derive(Debug)]
pub struct Transform {
    pub position: Point,
    pub rotation: Vector3,
    pub scale: Vector3,
    matrix: Matrix4
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Point::origin(),
            rotation: Vector3::zero(),
            scale:    Vector3::one(),
            matrix:   Matrix4::identity()
        }
    }

    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }

    // TODO: This shouldn't be a member of Transform, it should be done by the TransformManager.
    pub fn update(&mut self) {
        self.matrix = Matrix4::from_point(self.position)
                   * (Matrix4::rotation(self.rotation.x, self.rotation.y, self.rotation.z)
                    * Matrix4::scale(self.scale.x, self.scale.y, self.scale.z));
    }
}
