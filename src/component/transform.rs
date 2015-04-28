use std::collections::HashMap;
use std::cell::Cell;

use math::vector::Vector3;
use math::matrix::Matrix4;
use math::point::Point;
use ecs::{Entity, System, ComponentManager};
use scene::Scene;

pub struct TransformManager {
    transforms: Vec<Vec<Transform>>,
    entities: Vec<Vec<Entity>>,

    /// A map between the entity owning the transform and the location of the transform.
    ///
    /// The first value of the mapped tuple is the row containing the transform, the
    /// second is the index of the transform within that row.
    indices: HashMap<Entity, (usize, usize)>,
}

impl TransformManager {
    pub fn new() -> TransformManager {
        let mut transform_manager = TransformManager {
            transforms: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        };

        transform_manager.transforms.push(Vec::new());
        transform_manager.entities.push(Vec::new());
        transform_manager
    }

    pub fn create(&mut self, entity: Entity) -> &mut Transform {
        let index = self.transforms[0].len();
        self.transforms[0].push(Transform::new());
        self.entities[0].push(entity);

        assert!(self.transforms[0].len() == self.entities[0].len());

        self.indices.insert(entity, (0, index));
        &mut self.transforms[0][index]
    }

    pub fn get(&self, entity: Entity) -> &Transform {
        let (row, index) = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        &self.transforms[row][index]
    }

    pub fn get_mut(&mut self, entity: Entity) -> &mut Transform {
        let (row, index) = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        &mut self.transforms[row][index]
    }

    pub fn set_child(&mut self, parent: Entity, child: Entity) {
        // Remove old transform component.
        let mut transform = self.remove(child);

        // Get the indices of the parent.
        let (parent_row, _) = *self.indices.get(&parent).unwrap();
        let child_row = parent_row + 1;

        // Ensure that there are enough rows for the child.
        while self.transforms.len() < child_row + 1 {
            self.transforms.push(Vec::new());
            self.entities.push(Vec::new());
        }
        // Add the child to the correct row.
        transform.parent = Some(parent);
        let child_index = self.transforms[child_row].len();
        self.transforms[child_row].push(transform);
        self.entities[child_row].push(child);

        // Update the index map.
        self.indices.insert(child, (child_row, child_index));
    }

    fn remove(&mut self, entity: Entity) -> Transform {
        // Retrieve indices of removed entity and the one it's swapped with.
        let (row, index) = *self.indices.get(&entity)
            .expect("Transform manager does not contain a transform for the given entity.");
        assert!(self.transforms[row].len() == self.entities[row].len());

        // Remove transform and the associate entity.
        let transform = self.transforms[row].swap_remove(index);
        let removed_entity = self.entities[row].swap_remove(index);
        assert!(removed_entity == entity);
        // Remove mapping for the removed component.
        self.indices.remove(&entity);

        // Update the index mapping for the moved entity, but only if the one we removed
        // wasn't the only one in the row.
        if index != self.entities[row].len() {
            let moved_entity = self.entities[row][index];
            self.indices.insert(moved_entity, (row, index));
        }

        transform
    }
}

impl ComponentManager for TransformManager {
}

#[derive(Debug)]
pub struct Transform {
    parent: Option<Entity>,
    pub position: Point,
    pub rotation: Matrix4,
    pub scale: Vector3,
    matrix: Cell<Matrix4>,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            parent: None,
            position: Point::origin(),
            rotation: Matrix4::identity(),
            scale:    Vector3::one(),
            matrix:   Cell::new(Matrix4::identity())
        }
    }

    pub fn matrix(&self) -> Matrix4 {
        self.matrix.get()
    }

    pub fn normal_matrix(&self) -> Matrix4 {
        let inverse =
            Matrix4::scale(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z)
          * (self.rotation.transpose()
          *  Matrix4::translation(-self.position.x, -self.position.y, -self.position.z));

        inverse.transpose()
    }

    fn update(&self, parent_matrix: Matrix4) {
        self.matrix.set(
            parent_matrix
                * (Matrix4::from_point(self.position)
                *  (self.rotation
                *   Matrix4::scale(self.scale.x, self.scale.y, self.scale.z))));
    }

    pub fn rotation_matrix(&self) -> Matrix4 {
        self.rotation
    }

    pub fn look_at(&mut self, interest: Point, up: Vector3) {
        let forward = interest - self.position;
        let forward = forward.normalized();
        let up = up.normalized();

        let right = Vector3::cross(forward, up);
        let up = Vector3::cross(right, forward);

        let mut look_matrix = Matrix4::identity();

        look_matrix[(0, 0)] = right.x;
        look_matrix[(1, 0)] = right.y;
        look_matrix[(2, 0)] = right.z;

        look_matrix[(0, 1)] = up.x;
        look_matrix[(1, 1)] = up.y;
        look_matrix[(2, 1)] = up.z;

        look_matrix[(0, 2)] = -forward.x;
        look_matrix[(1, 2)] = -forward.y;
        look_matrix[(2, 2)] = -forward.z;

        self.rotation = look_matrix;
    }
}

pub struct TransformUpdateSystem;

impl System for TransformUpdateSystem {
    fn update(&mut self, scene: &mut Scene, _: f32) {
        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        for row in transform_manager.transforms.iter() {
            for transform in row.iter() {
                // Retrieve the parent's transformation matrix, using the identity
                // matrix if the transform has no parent.
                let parent_matrix = {
                    match transform.parent {
                        None => Matrix4::identity(),
                        Some(parent) => {
                            let (parent_row, parent_index) = *transform_manager.indices.get(&parent).unwrap();
                            let ref parent_transform = transform_manager.transforms[parent_row][parent_index];
                            parent_transform.matrix.get()
                        }
                    }
                };

                transform.update(parent_matrix);
            }
        }
    }
}
