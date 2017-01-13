extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_gl as gl;
extern crate stopwatch;

use bootstrap::window::*;
use gl::*;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::*;
use stopwatch::*;

fn main() {
    let mut window = Window::new("OpenGL performance test").unwrap();

    let mut times = Vec::with_capacity(10_000);

    let device_context = window.platform().device_context();
    let context = unsafe { gl::create_context(device_context).unwrap() };
    unsafe { gl::make_current(context); }

    let target_frame_time = Duration::new(0, 1_000_000_000 / 60);
    let mut frame_start = Instant::now();
    let start_time = frame_start;

    'outer: loop {
        {
            let _s = Stopwatch::new("Loop");

            {
                let _s = Stopwatch::new("Window messages");
                while let Some(message) = window.next_message() {
                    if let Message::Close = message { break 'outer; }
                }
            }

            {
                let _s = Stopwatch::new("Clear buffer");
                unsafe { gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth); }
            }

            {
                let _s = Stopwatch::new("Swap buffers");
                unsafe { gl::platform::swap_buffers(context); }
            }

            times.push(frame_start.elapsed());
        }

        // Determine the next frame's start time, dropping frames if we missed the frame time.
        while frame_start < Instant::now() {
            frame_start += target_frame_time;
        }

        // Now wait until we've returned to the frame cadence before beginning the next frame.
        while Instant::now() < frame_start {
            thread::sleep(Duration::new(0, 0));
        }
    }

    let run_duration = start_time.elapsed();
    let stats = stats::analyze(&*times, target_frame_time);

    println!("Performance statistics:");
    println!("  Duration: {} ({} frames)", PrettyDuration(run_duration), times.len());
    println!("  Min: {}", PrettyDuration(stats.min));
    println!("  Max: {}", PrettyDuration(stats.max));
    println!("  Mean: {}", PrettyDuration(stats.mean));
    println!("  Std: {}", PrettyDuration(stats.std));
    println!("  Long frames: {} ({:.2}%)", stats.long_frames, stats.long_frame_ratio * 100.0);

    let events_string = stopwatch::write_events_to_string();
    let mut out_file = File::create("bootstrap_gl_empty.json").unwrap();
    out_file.write_all(events_string.as_bytes()).unwrap();
}
