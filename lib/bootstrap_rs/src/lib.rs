#![feature(core, collections)]

extern crate gl;

#[cfg(target_family = "windows")]
mod windows;

#[cfg(target_family = "windows")]
pub use windows::init::init;

pub mod window;
pub mod gl_utils;
pub mod input;

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
