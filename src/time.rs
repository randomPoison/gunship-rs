//! Information about frame timing.
//!
//! By default the engine will run at 60 fps (giving a delta of 16.667 ms), but it will change
//! its fixed framerate if necessary. For example, if the game fails to meet 60 fps, the engine
//! will throttle down to 30 fps (with a delta of 33.333 ms) until it can return to 60 fps. The
//! time delta doesn't represent the exact amount of time it took to complete the last frame,
//! rather it gives the current locked framerate for the game. Therefore, game code can be
//! written with the assumption of a fixed time step (i.e. the delta will be the same
//! frame-to-frame) even if the exact time step may occaisonally change in practice.

use std::time::Duration;

/// Returns the exact time between frames.
///
/// See module documentation for more information about frame timing.
pub fn delta() -> Duration {
    Duration::new(1, 0) / 60
}

/// Returns the current time between frames in seconds.
///
/// See module documentation for more information about frame timing.
pub fn delta_f32() -> f32 {
    1.0 / 60.0
}
