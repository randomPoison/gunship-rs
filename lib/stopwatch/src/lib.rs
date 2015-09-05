#![feature(drain)]

extern crate bootstrap_rs as bootstrap;

use std::ptr;
use std::fs::OpenOptions;
use std::io::Write;

use bootstrap::time::Timer;

pub mod null;

/// A global access point for collecting logs. This allows client code to create stopwatches
/// anywhere without having to pass the Collector around.
static mut COLLECTOR: *mut Collector = 0 as *mut Collector;

pub struct Collector {
    logs: Vec<String>,
}

impl Collector {
    pub fn new() -> Result<Box<Collector>, ()> {
        unsafe {
            if !COLLECTOR.is_null() {
                return Err(());
            }
        }

        let mut boxed = Box::new(Collector {
            logs: Vec::new(),
        });

        unsafe {
            COLLECTOR = &mut *boxed;
        }

        Ok(boxed)
    }

    pub fn flush_to_file(&mut self, file_name: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name).unwrap();
        for log in self.logs.drain(0..) {
            writeln!(file, "{}", log).unwrap();
        }
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        unsafe { COLLECTOR = ptr::null_mut(); }
    }
}

pub struct Stopwatch {
    timer: Timer,
    start_time: i64,
    name: String,
}

impl Stopwatch {
    pub fn new() -> Stopwatch {
        assert!(unsafe { !COLLECTOR.is_null() }, "Cannot create a stopwatch until a collector has been made.");

        let timer = Timer::new();
        let start_time = timer.now();
        Stopwatch {
            timer: timer,
            start_time: start_time,
            name: String::new(),
        }
    }

    pub fn named(name: &str) -> Stopwatch {
        assert!(unsafe { !COLLECTOR.is_null() }, "Cannot create a stopwatch until a collector has been made.");

        let timer = Timer::new();
        let start_time = timer.now();
        Stopwatch {
            timer: timer,
            start_time: start_time,
            name: String::from(name),
        }
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        let log = format!("{}, {}", self.name, self.timer.elapsed_ms(self.start_time));
        unsafe {
            let collect = &mut *COLLECTOR;
            collect.logs.push(log);
        }
    }
}
