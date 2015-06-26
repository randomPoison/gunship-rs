#[derive(Debug, Clone, Copy)]
pub struct Timer;

impl Timer {
    pub fn new() -> Timer {
        println!("Timer::new() isn't implemented on linux");
        Timer
    }

    pub fn now(&self) -> i64 {
        println!("Timer::now() isn't implemented on linux");
        0
    }

    pub fn elapsed(&self, _start: i64) -> f32 {
        println!("Timer::elapsed() isn't implemented on linux");
        1.0 / 30.0
    }

    pub fn elapsed_ms(&self, _start: i64) -> f32 {
        println!("Timer::elapsed_ms() isn't implemented on linux");
        1.0 / 30.0 * 1000.0
    }
}
