pub use platform::input::{set_cursor_visibility, set_cursor_bounds, clear_cursor_bounds};

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ScanCode {
    A = 'A' as u32,
    B = 'B' as u32,
    C = 'C' as u32,
    D = 'D' as u32,
    E = 'E' as u32,
    F = 'F' as u32,
    G = 'G' as u32,
    H = 'H' as u32,
    I = 'I' as u32,
    J = 'J' as u32,
    K = 'K' as u32,
    L = 'L' as u32,
    M = 'M' as u32,
    N = 'N' as u32,
    O = 'O' as u32,
    P = 'P' as u32,
    Q = 'Q' as u32,
    R = 'R' as u32,
    S = 'S' as u32,
    T = 'T' as u32,
    U = 'U' as u32,
    V = 'V' as u32,
    W = 'W' as u32,
    X = 'X' as u32,
    Y = 'Y' as u32,
    Z = 'Z' as u32,
    Key0 = '0' as u32,
    Key1 = '1' as u32,
    Key2 = '2' as u32,
    Key3 = '3' as u32,
    Key4 = '4' as u32,
    Key5 = '5' as u32,
    Key6 = '6' as u32,
    Key7 = '7' as u32,
    Key8 = '8' as u32,
    Key9 = '9' as u32,

    // TODO: Are these reasonable values for these codes?
    // These values were taken from the values observed from keypresses on my windows keyboard.
    // They're convenient for now because it means I can just reinterpret the scancodes I get from
    // Windows, but I don't know if these values make sense in a cross-platform context.
    Space    = 32 as u32,
    F9       = 120 as u32,
    F10      = 121 as u32,
    F11      = 122 as u32,
    BackTick = 192 as u32,

    Unsupported,
}
