#![allow(bad_style)]

extern crate gdi32;
extern crate winapi;
extern crate user32;
extern crate kernel32;
extern crate winmm;

pub mod window;
pub mod input;
pub mod file;

pub trait ToCU16Str {
    fn to_c_u16(&self) -> Vec<u16>;
}

impl<'a> ToCU16Str for &'a str {
    fn to_c_u16(&self) -> Vec<u16> {
        self.encode_utf16().chain(Some(0).into_iter()).collect()
    }
}
