#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![feature(proc_macro)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate bootstrap_rs as bootstrap;
extern crate fiber;

use fiber::FiberId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::sync::Mutex;
use std::time::Duration;

#[cfg(target_os="windows")]
#[path="windows.rs"]
pub mod platform;

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

lazy_static! {
    static ref CONTEXT_MAP: Mutex<HashMap<FiberId, Context>> = Mutex::new(HashMap::with_capacity(1024));
    static ref EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());
}

/// Swaps the currently tracked execution context with the specified context.
pub fn switch_context(old: FiberId, new: FiberId) {
    with_context(|stack| {
        let timestamp = platform::timestamp();

        // If there are stopwatches on the stack then we need to end the flow event.
        if stack.len() > 0 {
            push_event(Event {
                name: stack[0].name,
                cat: String::new(),
                ph: "f",
                id: fiber::current().map(FiberId::primitive_id),
                ts: timestamp,
                tid: platform::thread_id(),
                pid: 0,
                bp: "e",
            });
        }

        // Push an end event for each of the time slices.
        for stopwatch in stack.iter().rev() {
            push_event(Event {
                name: stopwatch.name,
                cat: String::new(),
                ph: "E",
                id: fiber::current().map(FiberId::primitive_id),
                ts: timestamp,
                tid: platform::thread_id(),
                pid: 0,
                bp: "e",
            });
        }
    });

    let mut context_map = CONTEXT_MAP.lock().expect("Unable to acquire lock on context map");

    let new_context = context_map.remove(&new).unwrap_or(Context::new());
    let old_context = with_context(move |context| {
        let mut new_context = new_context;
        mem::swap(context, &mut new_context);
        new_context
    });

    context_map.insert(old, old_context);

    with_context(|stack| {
        let timestamp = platform::timestamp();

        // Push an end event for each of the time slices.
        for stopwatch in stack.iter() {
            push_event(Event {
                name: stopwatch.name,
                cat: String::new(),
                ph: "B",
                id: fiber::current().map(FiberId::primitive_id),
                ts: timestamp,
                tid: platform::thread_id(),
                pid: 0,
                bp: "e",
            });
        }

        // If there are stopwatches on the stack then we need to end the flow event.
        if stack.len() > 0 {
            push_event(Event {
                name: stack[0].name,
                cat: String::new(),
                ph: "s",
                id: fiber::current().map(FiberId::primitive_id),
                ts: timestamp,
                tid: platform::thread_id(),
                pid: 0,
                bp: "e",
            });
        }
    });
}

/// Writes the events history to a string.
pub fn write_events_to_string() -> String {
    let events = EVENTS.lock().expect("Events mutex got poisoned");
    serde_json::to_string(&*events).unwrap()
}

pub struct Stopwatch {
    name: &'static str,
}

impl Stopwatch {
    pub fn new(name: &'static str) -> Stopwatch {
        push_event(Event {
            name: name,
            cat: String::new(),
            ph: "B",
            id: fiber::current().map(FiberId::primitive_id),
            ts: platform::timestamp(),
            tid: platform::thread_id(),
            pid: 0, // TODO: Do we care about tracking process ID?
            bp: "e",
        });

        with_context(|stack| {
            // The first event on the stack also needs a flow event.
            if stack.len() == 0 {
                push_event(Event {
                    name: name,
                    cat: String::new(),
                    ph: "s",
                    id: fiber::current().map(FiberId::primitive_id),
                    ts: platform::timestamp(),
                    tid: platform::thread_id(),
                    pid: 0,
                    bp: "e",
                })
            }

            stack.push(StopwatchData { name: name });
        });

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
        with_context(|stack| {
            let stopwatch = stack.pop().expect("No stopwatch popped, stack is corrupted");
            assert_eq!(self.name, stopwatch.name, "Stack got corrupted I guess");

            if stack.len() == 0 {
                push_event(Event {
                    name: self.name,
                    cat: String::new(),
                    ph: "f",
                    id: fiber::current().map(FiberId::primitive_id),
                    ts: platform::timestamp(),
                    tid: platform::thread_id(),
                    pid: 0,
                    bp: "e",
                });
            }
        });

        push_event(Event {
            name: self.name,
            cat: String::new(),
            ph: "E",
            id: fiber::current().map(FiberId::primitive_id),
            ts: platform::timestamp(),
            tid: platform::thread_id(),
            pid: 0, // TODO: Do we care about tracking process ID?
            bp: "e",
        });
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

    id: Option<isize>,

    bp: &'static str,
}

fn push_event(event: Event) {
    let mut events = EVENTS.lock().expect("Events mutex got poisoned");
    events.push(event);
}

#[derive(Debug, Clone, Copy)]
struct StopwatchData {
    name: &'static str,
}

type Context = Vec<StopwatchData>;

fn with_context<F, T>(func: F) -> T
    where F: FnOnce(&mut Context) -> T
{
    CONTEXT.with(move |context_cell| {
        let mut context = context_cell.borrow_mut();
        func(&mut *context)
    })
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
