use ecs::System;
use scene::Scene;
use component::{StructComponentManager, TransformManager};

pub use polygon::light::Light;
pub use polygon::light::PointLight;
pub type LightManager = StructComponentManager<Light>;

pub struct LightUpdateSystem;

impl System for LightUpdateSystem {
    fn update(&mut self, scene: &mut Scene, _delta: f32) {
        let mut light_handle = scene.get_manager::<LightManager>();
        let mut light_manager = light_handle.get();

        let mut transorm_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transorm_handle.get();

        for (light, entity) in light_manager.iter_mut() {
            let light_transform = transform_manager.get(entity);
            match light {
                &mut Light::Point(ref mut point_light) => {
                    point_light.position = light_transform.position;
                }
            }
        }
    }
}
