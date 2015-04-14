// #[cfg(windows)]
// pub use windows::input::*;

#[derive(FromPrimitive, Debug)]
pub enum ScanCode {
    A = 'A' as isize,
    B = 'B' as isize,
    C = 'C' as isize,
    D = 'D' as isize,
    E = 'E' as isize,
    F = 'F' as isize,
    G = 'G' as isize,
    H = 'H' as isize,
    I = 'I' as isize,
    J = 'J' as isize,
    K = 'K' as isize,
    L = 'L' as isize,
    M = 'M' as isize,
    N = 'N' as isize,
    O = 'O' as isize,
    P = 'P' as isize,
    Q = 'Q' as isize,
    R = 'R' as isize,
    S = 'S' as isize,
    T = 'T' as isize,
    U = 'U' as isize,
    V = 'V' as isize,
    W = 'W' as isize,
    X = 'X' as isize,
    Y = 'Y' as isize,
    Z = 'Z' as isize,
    Key_0 = '0' as isize,
    Key_1 = '1' as isize,
    Key_2 = '2' as isize,
    Key_3 = '3' as isize,
    Key_4 = '4' as isize,
    Key_5 = '5' as isize,
    Key_6 = '6' as isize,
    Key_7 = '7' as isize,
    Key_8 = '8' as isize,
    Key_9 = '9' as isize,

    Unsupported
}
