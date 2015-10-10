use std::mem;

pub const RED:   Color = Color { r: 1.0, b: 0.0, g: 0.0, a: 1.0 };
pub const WHITE: Color = Color { r: 1.0, b: 1.0, g: 1.0, a: 1.0 };

#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    pub fn as_array(&self) -> &[f32; 4] {
        unsafe { mem::transmute(self) }
    }
}
