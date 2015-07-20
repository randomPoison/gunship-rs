#[derive(Debug, Clone, Copy)]
pub struct Timer;

impl Timer {
    pub fn new() -> Timer {
        Timer
    }

    pub fn now(&self) -> i64 {
        0
    }

    pub fn elapsed(&self, start: i64) -> f32 {
        1.0 / 30.0
    }

    pub fn elapsed_ms(&self, start: i64) -> f32 {
        1.0 / 30.0
    }
}
