use ecs::{Entity, System, ComponentManager};
use scene::Scene;

#[derive(Debug, Clone)]
pub struct ColliderManager {
    colliders: Vec<Collider>,
    entity:    Vec<Entity>,
}

impl ComponentManager for ColliderManager {
    fn destroy_all(&self, _entity: Entity) {
        unimplemented!();
    }

    fn destroy_marked(&mut self) {
        unimplemented!();
    }
}

#[derive(Debug, Clone)]
pub struct CollisionUpdateSystem;

impl System for CollisionUpdateSystem {
    fn update(&mut self, _scene: &Scene, _delta: f32) {

    }
}

#[derive(Debug, Clone, Copy)]
pub enum Collider {
    Sphere,
    AABB,
    OBB,
    ConvexHull,
    Mesh,
}
