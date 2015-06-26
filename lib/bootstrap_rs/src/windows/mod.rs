#![feature(collections)]
#![allow(bad_style)]

extern crate winapi;
extern crate user32;
extern crate kernel32;
extern crate gdi32;
extern crate opengl32;
extern crate winmm;

pub mod init;
pub mod window;
pub mod gl;
pub mod input;
pub mod time;
pub mod file;

// TODO: This shouldn't be needed, there should be some standard function for creating wide strings.
pub trait ToCU16Str {
    fn to_c_u16(&self) -> Vec<u16>;
}

impl<'a> ToCU16Str for &'a str {
    fn to_c_u16(&self) -> Vec<u16> {
        let mut t: Vec<u16> = self.utf16_units().collect();
        t.push(0u16);
        t
    }
}
