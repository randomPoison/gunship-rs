use std::ops::{Add, AddAssign, Sub};

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

#[derive(Debug, Clone)]
pub struct Timer;

impl Timer {
    pub fn new() -> Timer {
        Timer
    }

    pub fn now(&self) -> TimeMark {
        TimeMark(0)
    }

    /// Calculates the elapsed time, in milliseconds, since the specified start time.
    pub fn elapsed_ms(&self, _start: TimeMark) -> f32 {
        1.0 / 3.0 * 1000.0
    }

    // Calculates the elapsed time since the give start time, returning a high precision duration.
    pub fn elapsed(&self, _start: TimeMark) -> Duration {
        Duration(0)
    }

    pub fn duration_seconds(&self, _duration: Duration) -> f32 {
        1.0 / 3.0
    }

    pub fn duration_ms(&self, _duration: Duration) -> f32 {
        1.0 / 3.0 * 1000.0
    }
}
