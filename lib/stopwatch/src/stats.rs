use std::time::Duration;

// Calculate performance statistics.
// ============================================================================================
fn as_nanos(duration: Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64
}

fn from_nanos(nanos: u64) -> Duration {
    let secs = nanos / 1_000_000_000;
    let subsec_nanos = nanos % 1_000_000_000;
    Duration::new(secs, subsec_nanos as u32)
}

#[derive(Debug, Clone, Copy)]
pub struct Statistics {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub std: Duration,
    pub long_frames: usize,
    pub long_frame_ratio: f64,
}

pub fn analyze(frame_times: &[Duration], target_frame_time: Duration) -> Statistics {
    let mut min = frame_times[0];
    let mut max = frame_times[0];
    let mut total = Duration::new(0, 0);
    let mut long_frames = 0;

    for time in frame_times.iter().cloned() {
        total += time;
        if time < min { min = time; }
        if time > max { max = time; }
        if time > target_frame_time { long_frames += 1; }
    }

    let mean = total / frame_times.len() as u32;
    let total_sqr_deviation = frame_times.iter().cloned().fold(0, |total, time| {
        let diff = if time < mean { mean - time } else { time - mean };

        // Convert to nanos so that we can square and hope we don't overflow ¯\_(ツ)_/¯.
        let nanos = as_nanos(diff);
        let diff_sqr = nanos * nanos;

        total + diff_sqr
    });

    let std_dev = from_nanos(f64::sqrt(total_sqr_deviation as f64 / frame_times.len() as f64) as u64);
    let long_frame_ratio = long_frames as f64 / frame_times.len() as f64;

    Statistics {
        min: min,
        max: max,
        mean: mean,
        std: std_dev,
        long_frames: long_frames,
        long_frame_ratio: long_frame_ratio,
    }
}
