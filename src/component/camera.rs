use std::collections::HashMap;
use std::slice::{Iter, IterMut};

pub use polygon::camera::Camera;
use math::point::Point;
use math::matrix::Matrix4;

use ecs::Entity;

pub struct CameraManager {
    cameras: Vec<Camera>,
    entities: Vec<Entity>,
    indices: HashMap<Entity, usize>,
}

impl CameraManager {
    pub fn new() -> CameraManager {
        CameraManager {
            cameras: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn create(&mut self, entity: Entity, fov: f32, aspect: f32, near: f32, far: f32,) -> &mut Camera {
        assert!(!self.indices.contains_key(&entity));

        let index = self.cameras.len();
        self.cameras.push(Camera {
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,

            position: Point::origin(),
            rotation: Matrix4::identity()
        });
        self.entities.push(entity);
        self.indices.insert(entity, index);

        &mut self.cameras[index]
    }

    pub fn cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn cameras_mut(&mut self) -> &mut Vec<Camera> {
        &mut self.cameras
    }

    pub fn iter(&self) -> CameraIter {
        CameraIter {
            cameras_iter: self.cameras.iter(),
            entity_iter: self.entities.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> CameraIterMut {
        CameraIterMut {
            cameras_iter: self.cameras.iter_mut(),
            entity_iter: self.entities.iter(),
        }
    }
}

pub struct CameraIter<'a> {
    cameras_iter: Iter<'a, Camera>,
    entity_iter: Iter<'a, Entity>,
}

impl<'a> Iterator for CameraIter<'a> {
    type Item = (&'a Camera, Entity);

    fn next(&mut self) -> Option<(&'a Camera, Entity)> {
        match self.cameras_iter.next() {
            None => None,
            Some(camera) => Some((camera, *self.entity_iter.next().unwrap()))
        }
    }
}

pub struct CameraIterMut<'a> {
    cameras_iter: IterMut<'a, Camera>,
    entity_iter: Iter<'a, Entity>,
}


impl<'a> Iterator for CameraIterMut<'a> {
    type Item = (&'a mut Camera, Entity);

    fn next(&mut self) -> Option<(&'a mut Camera, Entity)> {
        match self.cameras_iter.next() {
            None => None,
            Some(camera) => Some((camera, *self.entity_iter.next().unwrap()))
        }
    }
}
