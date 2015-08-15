use windows::winapi::*;
use windows::kernel32;

pub struct Timer {
    _frequency: f32,
    one_over_freq: f32,
    one_over_freq_ms: f32,
}

impl Timer {
    pub fn new() -> Timer {
        let mut frequency: LONGLONG = 0;
        let result = unsafe {
            kernel32::QueryPerformanceFrequency(&mut frequency)
        };
        assert!(result != 0);

        Timer {
            _frequency: frequency as f32,
            one_over_freq: 1.0 / frequency as f32,
            one_over_freq_ms: 1.0 / frequency as f32 * 1000.0,
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
        elapsed_cycles as f32 * self.one_over_freq
    }

    /// Calculates the elapsed time, in milliseconds, since the specified start time.
    pub fn elapsed_ms(&self, start: i64) -> f32 {
        let now = self.now();
        let elapsed_cycles = now - start;
        elapsed_cycles as f32 * self.one_over_freq_ms
    }
}
