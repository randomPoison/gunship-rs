extern crate bootstrap_rs as bootstrap;

use std::cmp::Ordering;
use std::collections::HashMap;
// use std::fs::OpenOptions;
// use std::io::Write;
use std::ptr;

use bootstrap::time::{Timer, TimeMark};

pub mod null;

/// A global access point for collecting logs. This allows client code to create stopwatches
/// anywhere without having to pass the Collector around.
static mut COLLECTOR: *mut Collector = 0 as *mut Collector;

pub struct Collector {
    nodes:      Vec<StackNode>,
    call_stack: Vec<usize>,
}

impl Collector {
    pub fn new() -> Result<Box<Collector>, ()> {
        unsafe {
            if !COLLECTOR.is_null() {
                return Err(());
            }
        }

        let mut boxed = Box::new(Collector {
            nodes:      Vec::new(),
            call_stack: Vec::new(),
        });

        unsafe {
            COLLECTOR = &mut *boxed;
        }

        Ok(boxed)
    }

    pub fn flush_to_file(&mut self, _file_name: &str) {
        // TODO: Actually write to a file.
        // let mut file = OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .open(file_name).unwrap();
        // for log in self.logs.drain(0..) {
        //     writeln!(file, "{}", log).unwrap();
        // }

        // For now we're going to just print to the console.
        self.print_node(&self.nodes[0], 0);
    }

    fn print_node<'a>(&'a self, node: &'a StackNode, depth: usize) {
        for _ in 0..depth {
            print!("    ");
        }

        let mut sorted_data = node.data.clone();
        sorted_data.sort_by(|lhs, rhs| {
            if lhs < rhs {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let median = sorted_data[sorted_data.len() / 2];
        println!("{}: median {:.6}ms", node.name, median);

        for (_, child_index) in &node.children {
            self.print_node(&self.nodes[*child_index], depth + 1);
        }
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        unsafe { COLLECTOR = ptr::null_mut(); }
    }
}

pub struct Stopwatch {
    timer:      Timer,
    start_time: TimeMark,
    name:       &'static str,
}

impl Stopwatch {
    pub fn new(name: &'static str) -> Stopwatch {
        assert!(unsafe { !COLLECTOR.is_null() }, "Cannot create a stopwatch until a Collector has been made.");

        push_call_stack(name);

        let timer = Timer::new();
        let start_time = timer.now();
        Stopwatch {
            timer: timer,
            start_time: start_time,
            name: name,
        }
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        let elapsed = self.timer.elapsed_ms(self.start_time);
        pop_call_stack(self.name, elapsed);
    }
}

fn push_call_stack(name: &'static str) {
    unsafe {
        debug_assert!(!COLLECTOR.is_null(), "Cannot push call stack without a collector instance.");

        let collector = &mut *COLLECTOR;
        // Get the index of the node that's going to be the new top node.
        let top_index = match collector.call_stack.last() {
            Some(top_index) => {
                // TODO: Sucks that we have to re-borrow the top node a bunch of times to convince
                // the borrow checker we're in the clear, maybe revisit this once non-lexical
                // borrows drop.
                let has_child = {
                    let top_node = &mut collector.nodes[*top_index];
                    top_node.children.contains_key(name)
                };
                if !has_child {
                    // We need to add a new child node to the top node.
                    let new_child = StackNode::new(name);
                    let child_index = collector.nodes.len();
                    collector.nodes.push(new_child);
                    let mut top_node = &mut collector.nodes[*top_index];
                    top_node.children.insert(name, child_index);
                }

                let top_node = &mut collector.nodes[*top_index];
                *top_node.children.get(name).unwrap()
            },
            None => {
                // Create new stack node if there are none, otherwise the first node is the root.
                if collector.nodes.len() == 0 {
                    let node = StackNode::new(name);
                    collector.nodes.push(node);
                }
                0
            }
        };
        collector.call_stack.push(top_index);
    }
}

fn pop_call_stack<'a>(name: &'static str, duration: f32) {
    unsafe {
        debug_assert!(!COLLECTOR.is_null(), "Cannot pop call stack without a collector instance.");

        let collector = &mut *COLLECTOR;
        let node_index = match collector.call_stack.pop() {
            Some(index) => index,
            None => panic!("Tried to pop with node name {} but stack is empty", name),
        };
        let node = &mut collector.nodes[node_index];
        debug_assert_eq!(name, node.name);
        node.data.push(duration);
    }
}

struct StackNode {
    name:     &'static str,
    children: HashMap<&'static str, usize>,
    data:     Vec<f32>,
}

impl StackNode {
    fn new(name: &'static str) -> StackNode {
        StackNode {
            name:     name,
            children: HashMap::new(),
            data:     Vec::new(),
        }
    }
}
