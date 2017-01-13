extern crate kernel32;

/// Gets the current timestamp in microseconds.
pub fn timestamp() -> i64 {
    let mut frequency = 0;
    if unsafe { kernel32::QueryPerformanceFrequency(&mut frequency) } == 0 {
        panic!("Failed to query performance frequency");
    }

    let mut counter = 0;
    if unsafe { kernel32::QueryPerformanceCounter(&mut counter) } == 0 {
        panic!("Failed to query performance counter");
    }

    counter * 1_000_000 / frequency
}

pub fn thread_id() -> usize {
    unsafe { kernel32::GetCurrentThreadId() as usize }
}
