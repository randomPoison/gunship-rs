use std::ptr;

use windows::winapi::{HINSTANCE};
use windows::kernel32;

pub fn init() -> HINSTANCE {
    unsafe {
        kernel32::GetModuleHandleW(ptr::null()) as HINSTANCE
    }
}
