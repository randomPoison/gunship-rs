use super::Scene;

pub trait System {
    fn update(&mut self, scene: &mut Scene, delta: f32);
}
