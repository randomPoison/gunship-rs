use std::collections::HashMap;
use std::cell::Cell;

use math::Vector3;
use math::Matrix4;
use math::Point;
use math::Quaternion;

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

    pub fn update_single(&self, entity: Entity) {
        let transform = self.get(entity);
        self.update_transform(transform);
    }

    pub fn update_transform(&self, transform: &Transform) {
        let (parent_matrix, parent_rotation) = match transform.parent {
            None => {
                (Matrix4::identity(), Quaternion::identity())
            },
            Some(parent) => {
                let parent_transform = self.get(parent);

                if parent_transform.out_of_date.get() {
                    self.update_transform(parent_transform);
                }

                let parent_matrix = parent_transform.matrix_derived.get();
                let parent_rotation = parent_transform.rotation_derived.get();

                (parent_matrix, parent_rotation)
            }
        };

        transform.update(parent_matrix, parent_rotation);
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
    position:         Point,
    rotation:         Quaternion,
    scale:            Vector3,
    local_matrix:     Cell<Matrix4>,
    position_derived: Cell<Point>,
    rotation_derived: Cell<Quaternion>,
    scale_derived:    Cell<Vector3>,
    matrix_derived:   Cell<Matrix4>,
    parent:           Option<Entity>,
    out_of_date:      Cell<bool>,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position:         Point::origin(),
            rotation:         Quaternion::identity(),
            scale:            Vector3::one(),
            local_matrix:     Cell::new(Matrix4::identity()),
            position_derived: Cell::new(Point::origin()),
            rotation_derived: Cell::new(Quaternion::identity()),
            scale_derived:    Cell::new(Vector3::one()),
            matrix_derived:   Cell::new(Matrix4::identity()),
            parent:           None,
            out_of_date:      Cell::new(false),
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn set_position(&mut self, new_position: Point) {
        self.position = new_position;
        self.out_of_date.set(true);
    }

    pub fn rotation(&self) -> Quaternion {
        self.rotation
    }

    pub fn set_rotation(&mut self, new_rotation: Quaternion) {
        self.rotation = new_rotation;
        self.out_of_date.set(true);
    }

    pub fn scale(&self) -> Vector3 {
        self.scale
    }

    pub fn set_scale(&mut self, new_scale: Vector3) {
        self.scale = new_scale;
        self.out_of_date.set(true);
    }

    /// Retrieves the derived position of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn position_derived(&self) -> Point {
        assert!(!self.out_of_date.get());

        self.position_derived.get()
    }

    /// Retrieves the derived rotation of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn rotation_derived(&self) -> Quaternion {
        assert!(!self.out_of_date.get());

        self.rotation_derived.get()
    }

    /// Retrieves the derived scale of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn scale_derived(&self) -> Vector3 {
        assert!(!self.out_of_date.get());

        self.scale_derived.get()
    }

    pub fn local_matrix(&self) -> Matrix4 {
        if self.out_of_date.get() {
            let local_matrix =
                Matrix4::from_point(self.position)
                * (self.rotation.as_matrix() * Matrix4::from_scale_vector(self.scale));
            self.local_matrix.set(local_matrix);
        }

        self.local_matrix.get()
    }

    pub fn derived_matrix(&self) -> Matrix4 {
        assert!(!self.out_of_date.get());

        self.matrix_derived.get()
    }

    pub fn derived_normal_matrix(&self) -> Matrix4 {
        assert!(!self.out_of_date.get());

        let inverse =
            Matrix4::from_scale_vector(1.0 / self.scale_derived.get())
          * (self.rotation_derived.get().as_matrix().transpose()
          *  Matrix4::from_point(-self.position_derived.get()));

        inverse.transpose()
    }

    pub fn look_at(&mut self, interest: Point, up: Vector3) {
        let forward = interest - self.position;
        self.rotation = Quaternion::look_rotation(forward, up);
        self.out_of_date.set(true);
    }

    pub fn look_direction(&mut self, forward: Vector3, up: Vector3) {
        self.rotation = Quaternion::look_rotation(forward, up);
        self.out_of_date.set(true);
    }

    /// Updates the local and derived matrices for the transform.
    fn update(&self, parent_matrix: Matrix4, parent_rotation: Quaternion) {
        let local_matrix = self.local_matrix();

        let derived_matrix = parent_matrix * local_matrix;
        self.matrix_derived.set(derived_matrix);

        self.position_derived.set(derived_matrix.translation_part());
        self.rotation_derived.set(parent_rotation * self.rotation);

        self.out_of_date.set(false);
    }
}

pub struct TransformUpdateSystem;

impl System for TransformUpdateSystem {
    fn update(&mut self, scene: &mut Scene, _: f32) {
        let mut transform_handle = scene.get_manager::<TransformManager>();
        let transform_manager = transform_handle.get();

        for row in transform_manager.transforms.iter() {
            for transform in row.iter() {
                // Retrieve the parent's transformation matrix, using the identity
                // matrix if the transform has no parent.
                let (parent_matrix, parent_rotation) = match transform.parent {
                    None => {
                        (Matrix4::identity(), Quaternion::identity())
                    },
                    Some(parent) => {
                        let parent_transform = transform_manager.get(parent);

                        let parent_matrix = parent_transform.derived_matrix();
                        let parent_rotation = parent_transform.rotation_derived();

                        (parent_matrix, parent_rotation)
                    }
                };

                transform.update(parent_matrix, parent_rotation);
            }
        }
    }
}
