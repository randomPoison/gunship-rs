use windows::winapi::*;
use windows::kernel32;

pub struct Timer {
    frequency: f32,
}

impl Timer {
    pub fn new() -> Timer {
        let mut frequency: LONGLONG = 0;
        let result = unsafe {
            kernel32::QueryPerformanceFrequency(&mut frequency)
        };
        assert!(result != 0);

        Timer {
            frequency: frequency as f32,
        }
    }

    pub fn now(&self) -> i64 {
        let mut counter: LONGLONG = 0;
        let result = unsafe {
            kernel32::QueryPerformanceCounter(&mut counter)
        };
        assert!(result != 0);
        counter
    }

    /// Calculates the elapsed time, in seconds, since the specified start time.
    pub fn elapsed(&self, start: i64) -> f32 {
        let now = self.now();
        let elapsed_cycles = now - start;
        elapsed_cycles as f32 / self.frequency
    }

    /// Calculates the elapsed time, in milliseconds, since the specified start time.
    pub fn elapsed_ms(&self, start: i64) -> f32 {
        let now = self.now();
        let elapsed_cycles = now - start;
        elapsed_cycles as f32 / self.frequency * 1000.0
    }
}
