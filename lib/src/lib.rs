#![crate_type = "lib"]
#![crate_name = "bootstrap-rs"]
#![feature(core, collections, std_misc, alloc)]

extern crate winapi;
extern crate "user32-sys" as user32;
extern crate "kernel32-sys" as kernel32;
extern crate "gdi32-sys" as gdi32;
extern crate "opengl32-sys" as opengl32;

extern crate gl;

pub mod window;
pub mod gl_render;

use std::ptr;

// use kernel32;
use winapi::{HINSTANCE};

pub fn main_instance() -> HINSTANCE {
    unsafe {
        kernel32::GetModuleHandleW(ptr::null()) as HINSTANCE
    }
}

pub trait ToCU16Str {
    fn to_c_u16(&self) -> Vec<u16>;
}

impl<'a> ToCU16Str for &'a str {
    fn to_c_u16(&self) -> Vec<u16> {
        let mut t:Vec<u16> = self.utf16_units().collect();
        t.push(0u16);
        t
    }
}

impl ToCU16Str for String {
    fn to_c_u16(&self) -> Vec<u16> {
        self.as_slice().to_c_u16()
    }
}
