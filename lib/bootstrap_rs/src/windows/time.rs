use windows::winapi::*;
use windows::kernel32;

pub fn now() -> i64 {
    let mut counter: LONGLONG = 0;
    let result = unsafe {
        kernel32::QueryPerformanceCounter(&mut counter)
    };
    assert!(result != 0);
    counter
}

pub fn frequency() -> i64 {
    let mut frequency: LONGLONG = 0;
    let result = unsafe {
        kernel32::QueryPerformanceFrequency(&mut frequency)
    };
    assert!(result != 0);
    frequency
}
