use super::Engine;

pub trait System {
    fn update(&mut self, engine: &mut Engine, delta: f32);
}
