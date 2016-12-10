use cell_extras::atomic_init_cell::AtomicInitCell;
use cell_extras::atomic_ref_cell::AtomicRefCell;
use engine::{self, EngineMessage};
use math::*;
use polygon::light::*;
use std::mem;
use std::sync::Arc;

// TODO: This shouldn't be fully public, only public within the crate.
pub type LightInner = Arc<(AtomicInitCell<LightId>, AtomicRefCell<Light>)>;

#[derive(Debug)]
pub struct DirectionalLight {
    data: LightInner,
}

impl DirectionalLight {
    pub fn new(direction: Vector3, color: Color, strength: f32) -> DirectionalLight {
        let light = Light::directional(direction, strength, color);
        let data = Arc::new((AtomicInitCell::new(), AtomicRefCell::new(light)));
        engine::send_message(EngineMessage::Light(data.clone()));
        DirectionalLight {
            data: data,
        }
    }

    pub fn forget(self) {
        mem::forget(self);
    }
}

#[derive(Debug)]
pub struct PointLight {
    data: LightInner,
}

impl PointLight {
    pub fn new(radius: f32, color: Color, strength: f32) -> PointLight {
        let light = Light::point(radius, strength, color);
        let data = Arc::new((AtomicInitCell::new(), AtomicRefCell::new(light)));
        engine::send_message(EngineMessage::Light(data.clone()));
        PointLight {
            data: data,
        }
    }

    pub fn forget(self) {
        mem::forget(self);
    }
}
