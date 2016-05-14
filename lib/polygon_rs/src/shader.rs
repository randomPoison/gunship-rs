/// Identifies a shader that has been compiled and linked on the GPU.
///
/// Shaders are created by the renderer by compiling shader source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Shader(usize);
derive_Counter!(Shader);
