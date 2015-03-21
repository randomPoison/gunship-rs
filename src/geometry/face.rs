/// A struct representing a face on a mesh.
///
/// A face is represented by exactly 3 indices into an array of vertices. render-rs does not
/// yet (and likely never will) support faces with more than three vertices.
#[repr(C)] #[derive(Copy)]
pub struct Face {
    pub indices: [u32; 3]
}

/// Utility macro for easily defining hard-coded faces.
///
/// # Examples
///
/// ```
/// let face = face!(0, 1, 2);
///
/// // equivalent to:
/// let face = Face {
///     indices: [0, 1, 2]
/// };
/// ```
#[macro_export]
macro_rules! face {
    ( $a:expr, $b:expr, $c:expr ) => {
        Face {
            indices: [$a, $b, $c]
        }
    };
}
