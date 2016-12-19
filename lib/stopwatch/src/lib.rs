#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate bootstrap_rs as bootstrap;
extern crate cell_extras;
extern crate fiber;

use cell_extras::AtomicInitCell;
use fiber::FiberId;
use std::fmt::{self, Debug, Formatter};
use std::sync::{Mutex, Once, ONCE_INIT};
use std::time::Duration;

#[cfg(target_os="windows")]
#[path="windows.rs"]
pub mod platform;

static EVENTS: AtomicInitCell<Mutex<Vec<Event>>> = AtomicInitCell::new();
static EVENTS_INIT: Once = ONCE_INIT;

/// Swaps the currently tracked execution context with the specified context.
pub fn switch_context(_old: FiberId, _new: FiberId) {
    // TODO: Can we track context switches with the event system?
}

/// Writes the events history to a string.
pub fn write_events_to_string() -> String {
    let mutex = EVENTS.borrow();
    let events = mutex.lock().expect("Events mutex got poisoned");
    serde_json::to_string(&*events).unwrap()
}

pub struct Stopwatch {
    name: &'static str,
}

impl Stopwatch {
    pub fn new(name: &'static str) -> Stopwatch {
        // TODO: Maybe we shouldn't lazily init the stopwatch system. It's probably better to
        // explicitly init it at startup.
        EVENTS_INIT.call_once(|| {
            EVENTS.init(Mutex::new(Vec::new()));
        });

        let event = Event {
            name: name,
            cat: String::new(),
            ph: "b",
            id: fiber::current().unwrap().primitive_id(),
            ts: platform::timestamp(),
            tid: platform::thread_id(),
            pid: 0, // TODO: Do we care about tracking process ID?
        };

        push_event(event);

        Stopwatch {
            name: name,
        }
    }

    pub fn with_budget(name: &'static str, _budget: Duration) -> Stopwatch {
        // TODO: We should actually do something with the budget, right?
        Stopwatch::new(name)
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        let event = Event {
            name: self.name,
            cat: String::new(),
            ph: "e",
            id: fiber::current().unwrap().primitive_id(),
            ts: platform::timestamp(),
            tid: platform::thread_id(),
            pid: 0, // TODO: Do we care about tracking process ID?
        };

        push_event(event);
    }
}

#[derive(Debug, Serialize)]
struct Event {
    /// Human-readable name for the event.
    name: &'static str,

    /// Event category.
    cat: String,

    /// Event phase (i.e. the event type).
    ph: &'static str,

    /// Timestamp in microseconds.
    ts: i64,

    /// Process ID for the event.
    pid: usize,

    /// Thread ID for the event.
    tid: usize,

    id: isize,
}

fn push_event(event: Event) {
    let mutex = EVENTS.borrow();
    let mut events = mutex.lock().expect("Events mutex got poisoned");
    events.push(event);
}

pub struct PrettyDuration(pub Duration);

impl Debug for PrettyDuration {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        let secs = self.0.as_secs();
        let millis = (self.0.subsec_nanos() / 1_000_000) % 1_000;
        let micros = (self.0.subsec_nanos() / 1_000) % 1_000;
        if secs > 0 {
            write!(formatter, "{}s {}ms {}μs", secs, millis, micros)
        } else {
            write!(formatter, "{}ms {}μs", millis, micros)
        }
    }
}
