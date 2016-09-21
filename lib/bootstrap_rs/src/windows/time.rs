use std::ops::{Add, AddAssign, Sub};

use super::winapi::*;
use super::kernel32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeMark(i64);

impl Sub for TimeMark {
    type Output = Duration;

    fn sub(self, rhs: TimeMark) -> Duration {
        Duration(self.0 - rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(i64);

impl Duration {
    pub fn new() -> Duration {
        Duration(0)
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, other: Duration) -> Duration {
        Duration(self.0 + other.0)
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs.0;
    }
}

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

    pub fn now(&self) -> TimeMark {
        let mut counter: LONGLONG = 0;
        let result = unsafe {
            kernel32::QueryPerformanceCounter(&mut counter)
        };
        assert!(result != 0);
        TimeMark(counter)
    }

    /// Calculates the elapsed time, in seconds, since the specified start time.
    pub fn elapsed_seconds(&self, start: TimeMark) -> f32 {
        let now = self.now();
        self.duration_seconds(now - start)
    }

    /// Calculates the elapsed time, in milliseconds, since the specified start time.
    pub fn elapsed_ms(&self, start: TimeMark) -> f32 {
        let now = self.now();
        self.duration_ms(now - start)
    }

    // Calculates the elapsed time since the give start time, returning a high precision duration.
    pub fn elapsed(&self, start: TimeMark) -> Duration {
        let now = self.now();
        now - start
    }

    pub fn duration_seconds(&self, duration: Duration) -> f32 {
        let Duration(elapsed_cycles) = duration;
        elapsed_cycles as f32 * self.one_over_freq
    }

    pub fn duration_ms(&self, duration: Duration) -> f32 {
        let Duration(elapsed_cycles) = duration;
        elapsed_cycles as f32 * self.one_over_freq_ms
    }
}
