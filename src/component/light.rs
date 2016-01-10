use ecs::*;
use scene::Scene;
use component::{DefaultManager, TransformManager};

pub use polygon::light::Light;
pub use polygon::light::PointLight;

impl Component for Light {}

pub type LightManager = DefaultManager<Light>;

#[derive(Debug, Clone, Copy)]
pub struct LightUpdateSystem;

impl System for LightUpdateSystem {
    fn update(&mut self, scene: &Scene, _delta: f32) {
        let mut light_manager = unsafe { scene.get_manager_mut::<LightManager>() }; // FIXME: Is bad, use new system.
        let transform_manager = scene.get_manager::<TransformManager>();

        for (mut light, entity) in light_manager.iter_mut() {
            let light_transform = transform_manager.get(entity).unwrap(); // TODO: Don't panic.
            match &mut *light {
                &mut Light::Point(ref mut point_light) => {
                    point_light.position = light_transform.position();
                }
            }
        }
    }
}
