use ecs::System;
use scene::Scene;
use component::{StructComponentManager, TransformManager};

pub use polygon::light::Light;
pub use polygon::light::PointLight;
pub type LightManager = StructComponentManager<Light>;

#[derive(Debug, Clone, Copy)]
pub struct LightUpdateSystem;

impl System for LightUpdateSystem {
    fn update(&mut self, scene: &Scene, _delta: f32) {
        let light_manager = scene.get_manager::<LightManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (mut light, entity) in light_manager.iter_mut() {
            let light_transform = transform_manager.get(entity);
            match &mut *light {
                &mut Light::Point(ref mut point_light) => {
                    point_light.position = light_transform.position();
                }
            }
        }
    }
}
