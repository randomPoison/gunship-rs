use math::vector::Vector3;
use math::matrix::Matrix4;
use math::point::Point;
use entity::Entity;

pub struct TransformManager {
    transforms: Vec<Transform>
}

pub struct Transform {
    pub position: Point,
    pub rotation: Vector3,
    pub scale: Vector3,
    matrix: Matrix4
}

impl TransformManager {
    pub fn new() -> TransformManager {
        TransformManager {
            transforms: Vec::new()
        }
    }

    pub fn create(&mut self, entity: Entity) -> &mut Transform {
        let index = self.transforms.len();
        self.transforms.push(Transform::new());
        &mut self.transforms[index]
    }
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
