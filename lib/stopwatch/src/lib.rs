extern crate bootstrap_rs as bootstrap;
extern crate fiber;
#[macro_use]
extern crate lazy_static;

use fiber::FiberId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::sync::Mutex;
use std::time::{Duration, Instant};

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context { stack: Vec::new() });
}

lazy_static! {
    static ref CONTEXT_MAP: Mutex<HashMap<FiberId, Context>> = Mutex::new(HashMap::with_capacity(1024));
}

/// Swaps the currently tracked execution context with the specified context.
pub fn switch_context(old: FiberId, new: FiberId) {
    let mut context_map = CONTEXT_MAP.lock().expect("Unable to acquire lock on context map");

    let new_context = context_map.remove(&new).unwrap_or(Context { stack: Vec::new() });
    let old_context = with_context(move |context| {
        let mut new_context = new_context;
        mem::swap(context, &mut new_context);
        new_context
    });

    context_map.insert(old, old_context);
}

pub struct Stopwatch {
    name: &'static str,
}

impl Stopwatch {
    pub fn new(name: &'static str) -> Stopwatch {
        let data = StopwatchData {
            name: name,
            start_time: Instant::now(),
            budget: None,
            children: Vec::new(),
        };

        with_context(move |context| context.push(data));

        Stopwatch {
            name: name,
        }
    }

    pub fn with_budget(name: &'static str, budget: Duration) -> Stopwatch {
        let data = StopwatchData {
            name: name,
            start_time: Instant::now(),
            budget: Some(budget),
            children: Vec::new(),
        };

        with_context(move |context| context.push(data));

        Stopwatch {
            name: name,
        }
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        with_context(|context| context.pop(self));
    }
}

#[derive(Debug)]
struct Context {
    stack: Vec<StopwatchData>,
}

impl Context {
    fn push(&mut self, data: StopwatchData) {
        self.stack.push(data);
    }

    fn pop(&mut self, current: &Stopwatch) {
        let end_time = Instant::now();

        let data = self.stack
            .pop()
            .unwrap_or_else(|| panic!("Stopwatch stack corrupted, no stopwatches left to pop, trying to pop \"{}\"", current.name));
        assert_eq!(data.name, current.name, "Stopwatch stack corrupted, mismatched stopwatch popped");

        let record = StopwatchRecord {
            name: data.name,
            start_time: data.start_time,
            end_time: end_time,
            budget: data.budget,
            children: data.children,
        };

        // If the budget was exceeded, then log record and its child hierarchy.
        if let Some(budget) = record.budget {
            assert!(record.duration().as_secs() == 0, "TODO: Support durations greater than 1 second");

            if record.duration() > budget {
                // TODO: What's a better way for stopwatch to handle logging? We don't want to
                // print to stdout, but we don't necessarily know what logging method a client
                // crate would want to use.
                println!(
                    "Budget for {} exceeded: Budget was {:?}, actual was {:?} (exceeded by {:?})",
                    record.name,
                    PrettyDuration(budget),
                    PrettyDuration(record.duration()),
                    PrettyDuration(record.duration() - budget));

                log_timing_hierarchy(&record, 1, record.duration());
            }
        }

        // Add record to parent's list of children if necessary.
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(record);
        }
    }
}

#[derive(Debug)]
struct StopwatchData {
    name: &'static str,
    start_time: Instant,
    budget: Option<Duration>,
    children: Vec<StopwatchRecord>,
}

#[derive(Debug)]
struct StopwatchRecord {
    name: &'static str,
    start_time: Instant,
    end_time: Instant,
    budget: Option<Duration>,
    children: Vec<StopwatchRecord>,
}

impl StopwatchRecord {
    fn duration(&self) -> Duration {
        self.end_time - self.start_time
    }
}

fn with_context<F, T>(func: F) -> T
    where F: FnOnce(&mut Context) -> T
{
    CONTEXT.with(move |context_cell| {
        let mut context = context_cell.borrow_mut();
        func(&mut *context)
    })
}

fn log_timing_hierarchy(record: &StopwatchRecord, depth: usize, root_time: Duration) {
    let record_time = record.duration();

    let record_percent = (record_time.subsec_nanos() as f32 / root_time.subsec_nanos() as f32) * 100.0;

    // Calculate self time.
    let child_time = record.children
        .iter()
        .fold(Duration::from_millis(0), |total, child| total + child.duration());
    let self_time = record_time - child_time;
    let self_percent = (self_time.subsec_nanos() as f32 / root_time.subsec_nanos() as f32) * 100.0;

    // Left-pad current record.
    // TODO: Use http://www.leftpad.io/ for this so that we don't have to maintain our own
    // left-pad implementation.
    for _ in 0..depth {
        print!("  ");
    }

    // Print current record.
    println!("{}: {:?} ({:.1}%)", record.name, PrettyDuration(record_time), record_percent);

    if record.children.len() > 0 {
        // Left-pad "self" entry
        for _ in 0..depth + 1 {
            print!("  ");
        }

        println!("Self: {:?} ({:.1}%)", PrettyDuration(self_time), self_percent);

        // Log children recursively.
        for child in &record.children {
            log_timing_hierarchy(child, depth + 1, root_time);
        }
    }
}

pub struct PrettyDuration(pub Duration);

impl Debug for PrettyDuration {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        let millis = (self.0.subsec_nanos() / 1_000_000) % 1_000;
        let micros = (self.0.subsec_nanos() / 1_000) % 1_000;
        write!(formatter, "{}ms {}μs", millis, micros)
    }
}
