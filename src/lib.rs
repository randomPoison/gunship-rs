// Using `RawVec<T>`, could be replaced.
#![feature(alloc)]

// Almost certainly going to be stabilized as-is, unlikely to break anything.
#![feature(const_fn)]

// The scheduler puts a `Condvar` and `Mutex` into some statics.
#![feature(drop_types_in_const)]

// Used by the scheduler for handling work. We might be able to remove that with some unsafe magic,
// but even then being able to box a `FnOnce()` is valuable, so this is unlikely to go away.
#![feature(fnbox)]

// Useful when sending raw pointers between threads, could be replaced.
#![feature(unique)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_audio as bs_audio;
extern crate cell_extras;
extern crate fiber;
extern crate hash;
#[macro_use]
extern crate lazy_static;
extern crate parse_obj as obj;
extern crate polygon;

pub extern crate polygon_math as math;
pub extern crate stopwatch;

#[macro_use]
pub mod macros;

pub mod camera;
pub mod collections;
pub mod engine;
pub mod input;
pub mod light;
pub mod mesh_renderer;
pub mod prelude;
pub mod resource;
pub mod scheduler;
pub mod time;
pub mod transform;
