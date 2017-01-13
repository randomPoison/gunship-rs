use std::slice;

pub const RED:   Color = Color { r: 1.0, b: 0.0, g: 0.0, a: 1.0 };
pub const WHITE: Color = Color { r: 1.0, b: 1.0, g: 1.0, a: 1.0 };
pub const BLUE:  Color = Color { r: 0.0, b: 1.0, g: 0.0, a: 1.0 };

/// A struct representing a color.
///
/// Colors have a red, green, blue, and alpha component. If alpha is not needed used
/// `Color::rgb()` and the alpha component will default to `1.0`, effectively behaving as if there
/// were no alpha. Color components are represented in linear color space. If non-linear color
/// computations are needed use one of the other color types.
///
/// TODO: Add other color types -___-;
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Constructs a new `Color` from a red, green, blue, and alpha component.
    ///
    /// If alpha isn't needed use `Color::rbg()` to construct a `Color` object with the default
    /// alpha of `1.0`.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    /// Constructs a new `Color` from a red, green, and blue component.
    ///
    /// If alpha is needed use `Color::new()` to specify an alpha value.
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: 1.0,
        }
    }

    pub fn as_slice_of_arrays(colors: &[Color]) -> &[[f32; 4]] {
        let ptr = colors.as_ptr() as *const _;
        unsafe { slice::from_raw_parts(ptr, colors.len()) }
    }
}

impl Default for Color {
    fn default() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl From<[f32; 3]> for Color {
    fn from(from: [f32; 3]) -> Color {
        let [r, g, b] = from;
        Color {
            r: r,
            g: g,
            b: b,
            a: 1.0,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(from: [f32; 4]) -> Color {
        let [r, g, b, a] = from;
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(from: (f32, f32, f32)) -> Color {
        let (r, g, b) = from;
        Color {
            r: r,
            g: g,
            b: b,
            a: 1.0,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(from: (f32, f32, f32, f32)) -> Color {
        let (r, g, b, a) = from;
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(from: Color) -> [f32; 4] {
        let Color { r, g, b, a } = from;
        [r, g, b, a]
    }
}

impl<'a> From<&'a Color> for [f32; 4] {
    fn from(from: &Color) -> [f32; 4] {
        let &Color { r, g, b, a } = from;
        [r, g, b, a]
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(from: Color) -> (f32, f32, f32, f32) {
        let Color { r, g, b, a } = from;
        (r, g, b, a)
    }
}

impl<'a> From<&'a Color> for (f32, f32, f32, f32) {
    fn from(from: &Color) -> (f32, f32, f32, f32) {
        let &Color { r, g, b, a } = from;
        (r, g, b, a)
    }
}

impl AsRef<[f32]> for Color {
    fn as_ref(&self) -> &[f32] {
        let ptr = self as *const Color as *const f32;
        unsafe { slice::from_raw_parts(ptr, 4) }
    }
}
