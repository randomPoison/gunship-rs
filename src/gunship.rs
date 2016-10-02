#![feature(alloc)]
#![feature(associated_type_defaults)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(question_mark)]
#![feature(unboxed_closures)]
#![feature(unique)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_audio as bs_audio;
extern crate fiber;
extern crate hash;
#[macro_use]
extern crate lazy_static;
extern crate polygon;

pub extern crate polygon_math as math;

pub mod stopwatch {
    extern crate stopwatch;

    #[cfg(feature="timing")]
    pub use self::stopwatch::{Collector, Stopwatch};

    #[cfg(not(feature="timing"))]
    pub use self::stopwatch::null::{Collector, Stopwatch};
}

#[macro_use]
pub mod macros;

pub mod async;
