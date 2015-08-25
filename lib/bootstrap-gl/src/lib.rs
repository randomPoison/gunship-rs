#![feature(convert)]

extern crate bootstrap_rs as bootstrap;

#[cfg(target_os="windows")]
#[path="windows\\mod.rs"]
pub mod platform;

#[cfg(target_os = "linux")]
#[path="linux/mod.rs"]
pub mod platform;

use std::cell::Cell;
use std::mem;
use std::fmt::{self, Debug, Formatter};
use std::slice;
use std::str;
use std::ops::{Deref, BitOr};
use std::ptr;
use std::ffi::CStr;

use bootstrap::window::Window;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boolean {
    False = 0,
    True = 1,
}

pub type Byte = i8;
pub type UByte = u8;
pub type Short = i16;
pub type UShort = u16;
pub type Int = i32;
pub type UInt = u32;
pub type Fixed = i32;
pub type Int64 = i64;
pub type UInt64 = u64;
pub type SizeI = i32;
pub type Enum = u32;
pub type IntPtr = isize;
pub type SizeIPtr = usize;
pub type Sync = usize;
pub type BitField = u32;
pub type Half = u16;
pub type Float = f32;
pub type ClampF = f32;
pub type Double = f64;
pub type ClampD = f64;

#[allow(non_camel_case_types)]
pub type f16 = u16;

/// TODO: Use NonZero here so that Option<VertexArrayObject>::None can be used instead of 0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexArrayObject(u32);

impl VertexArrayObject {
    pub fn null() -> VertexArrayObject {
        VertexArrayObject(0)
    }
}

/// TODO: Use NonZero here so that Option<VertexBufferObject>::None can be used instead of 0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexBufferObject(u32);

impl VertexBufferObject {
    pub fn null() -> VertexBufferObject {
        VertexBufferObject(0)
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerCapability {
    Fog                   = 0x0B60,
    Lighting              = 0x0B50,
    Texture2D             = 0x0DE1,
    CullFace              = 0x0B44,
    AlphaTest             = 0x0BC0,
    Blend                 = 0x0BE2,
    ColorLogicOp          = 0x0BF2,
    Dither                = 0x0BD0,
    StencilTest           = 0x0B90,
    DepthTest             = 0x0B71,
    PointSmooth           = 0x0B10,
    LineSmooth            = 0x0B20,
    ScissorTest           = 0x0C11,
    ColorMaterial         = 0x0B57,
    Normalize             = 0x0BA1,
    RescaleNormal         = 0x803A,
    PolygonOffsetFill     = 0x8037,
    VertexArray           = 0x8074,
    NormalArray           = 0x8075,
    ColorArray            = 0x8076,
    TextureCoordArray     = 0x8078,
    Multisample           = 0x809D,
    SampleAlphaToCoverage = 0x809E,
    SampleAlphaToOne      = 0x809F,
    SampleCoverage        = 0x80A0,

    // OpenGL 4.3
    DebugOutput           = 0x92E0,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NoError          = 0,
    InvalidEnum      = 0x0500,
    InvalidValue     = 0x0501,
    InvalidOperation = 0x0502,
    StackOverflow    = 0x0503,
    StackUnderflow   = 0x0504,
    OutOfMemory      = 0x0505,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub platform_context: platform::Context,
    loader: Loader,
}

impl Context {
    pub fn new(window: &Window) -> Context {
        platform::init(window);
        let context = platform::create_context(window);

        Context {
            platform_context: context,
            loader: Loader::new(),
        }
    }

    pub fn swap_buffers(&self, window: &Window) {
        platform::swap_buffers(window);
    }

    pub fn gen_vertex_array(&self, array: &mut VertexArrayObject) {
        self.loader.gen_vertex_arrays(1, array);
    }

    pub fn gen_vertex_arrays(&self, arrays: &mut [VertexArrayObject]) {
        self.loader.gen_vertex_arrays(
            arrays.len() as i32,
            &mut arrays[0],
        );
    }

    pub fn gen_buffer(&self, buffer: &mut VertexBufferObject) {
        self.loader.gen_buffers(1, buffer);
    }

    pub fn gen_buffers(&self, buffers: &mut [VertexBufferObject]) {
        self.loader.gen_buffers(
            buffers.len() as i32,
            &mut buffers[0],
        );
    }

    pub fn buffer_data<T>(&self, target: BufferTarget, data: &[T], usage: BufferUsage) {
        self.loader.buffer_data(
            target,
            (data.len() * mem::size_of::<T>()) as isize,
            unsafe { mem::transmute(&data[0]) },
            usage,
        );
    }

    pub fn shader_source(&self, shader: ShaderObject, source: &str) {
        // No need to null terminate because we can tell OpenGL how long the string is.
        let temp_ptr = &source.as_bytes()[0];
        let len = source.len() as i32;
        self.loader.shader_source(
            shader,
            1,
            unsafe { mem::transmute(&temp_ptr) },
            &len);
    }

    pub fn get_shader_type(&self, shader: ShaderObject) -> Result<ShaderType, String> {
        let mut status = 0;
        self.loader.get_shader_param(shader, ShaderParam::ShaderType, &mut status);
        if status == 0 {
            Err(String::from("Failed to get shader type, check that the shader object provided is valid."))
        } else {
            Ok(unsafe {
                mem::transmute(status)
            })
        }
    }

    pub fn compile_shader(&self, shader: ShaderObject) -> Result<(), String> {
        self.loader.compile_shader(shader);

        let mut status = 0;
        self.loader.get_shader_param(shader, ShaderParam::CompileStatus, &mut status);

        if status == 0 {
            let mut len = 0;
            self.loader.get_shader_param(shader, ShaderParam::InfoLogLength, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            unsafe {
                // Subtract 1 to skip the trailing null character.
                buf.set_len((len as usize) - 1);
            }
            self.loader.get_shader_info_log(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr());
            Err(String::from(match str::from_utf8(buf.as_slice()) {
                Err(_) => "Shader info log not valid utf8",
                Ok(info_log) => info_log,
            }))
        } else {
            Ok(())
        }
    }

    pub fn link_program(&self, program: ProgramObject) -> Result<(), String> {
        self.loader.link_program(program);

        // Get the link status
        let mut status = 0;
        self.loader.get_program_param(program, ProgramParam::LinkStatus, &mut status);

        if status == 0 {
            let mut len = 0;
            self.loader.get_program_param(program, ProgramParam::InfoLogLength, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            unsafe {
                buf.set_len((len as usize) - 1); // Subtract 1 to skip the trailing null character.
            }
            self.loader.get_program_info_log(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr());
            Err(String::from(match str::from_utf8(buf.as_slice()) {
                Err(_) => "Program info log not valid utf8",
                Ok(info_log) => info_log,
            }))
        } else {
            Ok(())
        }
    }

    pub fn get_attrib(&self, program: ProgramObject, attrib: &[u8]) -> Option<AttributeLocation> {
        let attrib_location =
            self.loader.get_attrib_location(program, attrib.as_ptr() as *const _);
        if attrib_location == -1 {
            None
        } else {
            Some(AttributeLocation(attrib_location as u32))
        }
    }

    pub fn get_uniform(&self, program: ProgramObject, uniform: &[u8]) -> Option<UniformLocation> {
        let uniform_location =
            self.loader.get_uniform_location(program, uniform.as_ptr() as *const _);
        if uniform_location == -1 {
            None
        } else {
            Some(UniformLocation(uniform_location as u32))
        }
    }

    pub fn uniform_matrix_4x4(
        &self,
        uniform: UniformLocation,
        transpose: bool,
        matrix: &[f32; 16])
    {
        self.loader.uniform_matrix_4fv(
            uniform,
            1,
            transpose,
            matrix.as_ptr());
    }

    pub fn uniform_4f(&self, uniform: UniformLocation, data: &[f32; 4]) {
        self.loader.uniform_4fv(uniform, 1, data.as_ptr());
    }

    pub fn unbind_vertex_array(&self) {
        self.loader.bind_vertex_array(VertexArrayObject::null());
    }

    pub fn unbind_buffer(&self, target: BufferTarget) {
        self.loader.bind_buffer(target, VertexBufferObject::null());
    }
}

impl Deref for Context {
    type Target = Loader;

    fn deref<'a>(&'a self) -> &'a Loader {
        &self.loader
    }
}

/// TODO: Mark all functions as unsafe? Some of them are safe, though, and we don't want to have
///       to rewrite the entire interface in Context.
macro_rules! gen_proc_loader {
    ( $( $gl_proc:ident : fn $proc_name:ident( $( $arg:ident : $arg_ty:ty ),* ) $( -> $result:ty )*, )* ) => {
        pub struct Loader {
            proc_loader: fn(&str) -> Option<extern "C" fn()>,
            $(
                $proc_name: Cell<Option<extern "C" fn( $(
                    $arg_ty,
                )* ) $( -> $result )*>>,
            )*
        }

        impl Loader {
            pub fn new() -> Loader {
                Loader {
                    proc_loader: platform::proc_loader,
                    $(
                        $proc_name: Cell::new(None),
                    )*
                }
            }

            $(
                pub fn $proc_name(&self, $( $arg: $arg_ty, )* ) $( -> $result )* {
                    if let None = self.$proc_name.get() {
                        // println!(concat!("loading ", stringify!($gl_proc), "() for the first time."));

                        let $proc_name = (self.proc_loader)(stringify!($gl_proc));
                        self.$proc_name.set(unsafe {
                            mem::transmute($proc_name)
                        });
                    }
                    assert!(self.$proc_name.get().is_some());

                    // println!(concat!("calling ", stringify!($gl_proc), "()"));
                    (self.$proc_name.get().unwrap())($( $arg ),*)
                }
            )*
        }
    }
}

gen_proc_loader! {
    glEnable:
        fn enable(capability: ServerCapability),
    glDisable:
        fn disable(capability: ServerCapability),
    glClearColor:
        fn clear_color(red: f32, green: f32, blue: f32, alpha: f32),
    glDebugMessageCallback:
        fn debug_message_callback(
            callback: extern "C" fn(DebugSource, DebugType, UInt, DebugSeverity, SizeI, *const u8, *mut ()),
            user_param: *mut ()
        ),
    glGetError:
        fn get_error() -> ErrorCode,
    glViewport:
        fn viewport(x: i32, y: i32, width: i32, height: i32),
    glGenVertexArrays:
        fn gen_vertex_arrays(num_arrays: i32, arrays: *mut VertexArrayObject),
    glBindVertexArray:
        fn bind_vertex_array(vao: VertexArrayObject),
    glGenBuffers:
        fn gen_buffers(num_buffers: i32, buffers: *mut VertexBufferObject),
    glBindBuffer:
        fn bind_buffer(target: BufferTarget, buffer: VertexBufferObject),
    glBufferData:
        fn buffer_data(target: BufferTarget, size: isize, data: *const (), usage: BufferUsage),
    glClear:
        fn clear(mask: ClearBufferMask),
    glCreateShader:
        fn create_shader(shader_type: ShaderType) -> ShaderObject,
    glShaderSource:
        fn shader_source(
            shader: ShaderObject,
            count: i32,
            strings: *const *const u8,
            length: *const i32),
    glCompileShader:
        fn compile_shader(shader: ShaderObject),
    glGetShaderiv:
        fn get_shader_param(shader: ShaderObject, param_type: ShaderParam, param_out: *mut i32),
    glGetShaderInfoLog:
        fn get_shader_info_log(
            shader: ShaderObject,
            max_length: i32,
            length_out: *mut i32,
            log_out: *mut u8),
    glCreateProgram:
        fn create_program() -> ProgramObject,
    glAttachShader:
        fn attach_shader(program: ProgramObject, shader: ShaderObject),
    glLinkProgram:
        fn link_program(program: ProgramObject),
    glGetProgramiv:
        fn get_program_param(
            program: ProgramObject,
            param_type: ProgramParam,
            param_out: *mut i32),
    glGetProgramInfoLog:
        fn get_program_info_log(
            program: ProgramObject,
            max_length: i32,
            length_out: *mut i32,
            log_out: *mut u8),
    glUseProgram:
        fn use_program(program: ProgramObject),
    glGetAttribLocation:
        fn get_attrib_location(program: ProgramObject, attrib_name: *const i8) -> i32,
    glVertexAttribPointer:
        fn vertex_attrib_pointer(
            attrib: AttributeLocation,
            size: i32,
            gl_type: GLType,
            normalized: bool,
            stride: i32,
            offset: usize),
    glEnableVertexAttribArray:
        fn enable_vertex_attrib_array(attrib: AttributeLocation),
    glGetUniformLocation:
        fn get_uniform_location(program: ProgramObject, uniform_name: *const i8) -> i32,
    glUniformMatrix4fv:
        fn uniform_matrix_4fv(
            uniform: UniformLocation,
            count: i32,
            transpose: bool,
            values: *const f32),
    glUniform4fv:
        fn uniform_4fv(uniform: UniformLocation, count: i32, data: *const f32),
    glDrawElements:
        fn draw_elements(mode: DrawMode, count: i32, index_type: IndexType, offset: usize),
    glDepthFunc:
        fn depth_func(func: Comparison),
    glBlendFunc:
        fn blend_func(src_factor: SourceFactor, dest_factor: DestFactor),
}

impl Debug for Loader {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str("Loader").unwrap();
        Ok(())
    }
}

impl Clone for Loader {
    fn clone(&self) -> Loader {
        Loader::new()
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferTarget {
    ArrayBuffer             = 0x8892,
    AtomicCounterBuffer     = 0x92C0,
    CopyReadBuffer          = 0x8F36,
    CopyWriteBuffer         = 0x8F37,
    UniformBuffer           = 0x8A11,
    DispatchIndirectBuffer  = 0x90EE,
    DrawIndirectBuffer      = 0x8F3F,
    ElementArrayBuffer      = 0x8893,
    PixelPackBuffer         = 0x88EB,
    PixelUnpackBuffer       = 0x88EC,
    QueryBuffer             = 0x9192,
    ShaderStorageBuffer     = 0x90D2,
    TextureBuffer           = 0x8C2A,
    TransformFeedbackBuffer = 0x8C8E,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    StreamDraw  = 0x88E0,
    StreamRead  = 0x88E1,
    StreamCopy  = 0x88E2,
    StaticDraw  = 0x88E4,
    StaticRead  = 0x88E5,
    StaticCopy  = 0x88E6,
    DynamicDraw = 0x88E8,
    DynamicRead = 0x88E9,
    DynamicCopy = 0x88EA,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    ComputeShader        = 0x91B9,
    FragmentShader       = 0x8B30,
    VertexShader         = 0x8B31,
    GeometryShader       = 0x8DD9,
    TessEvaluationShader = 0x8E87,
    TessControlShader    = 0x8E88,
}

/// TODO: Use NonZero here so that Option<ShaderObject>::None can be used instead of 0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderObject(u32);

/// TODO: Use NonZero here so that Option<ProgramObject>::None can be used instead of 0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramObject(u32);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeLocation(u32);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformLocation(u32);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLType {
    Byte          = 0x1400,
    UnsignedByte  = 0x1401,
    Short         = 0x1402,
    UnsignedShort = 0x1403,
    Float         = 0x1406,
    Fixed         = 0x140C,
    Int           = 0x1404,
    UnsignedInt   = 0x1405,
    HalfFloat     = 0x140B,
    Double        = 0x140A,
    // GL_INT_2_10_10_10_REV
    // GL_UNSIGNED_INT_2_10_10_10_REV
    // GL_UNSIGNED_INT_10F_11F_11F_REV
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    UnsignedByte  = 0x1401,
    UnsignedShort = 0x1403,
    UnsignedInt   = 0x1405,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderParam {
    ShaderType         = 0x8B4F,
    DeleteStatus       = 0x8B80,
    CompileStatus      = 0x8B81,
    InfoLogLength      = 0x8B84,
    ShaderSourceLength = 0x8B88,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramParam {
    DeleteStatus             = 0x8B80,
    LinkStatus               = 0x8B82,
    ValidateStatus           = 0x8B83,
    InfoLogLength            = 0x8B84,
    AttachedShaders          = 0x8B85,
    ActiveUniforms           = 0x8B86,
    ActiveUniformMaxLength   = 0x8B87,
    ActiveAttributes         = 0x8B89,
    ActiveAttributeMaxLength = 0x8B8A,
}

/// TODO: Custom derive for Debug to show which flags are set.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearBufferMask {
    Depth   = 0x00000100,
    Stencil = 0x00000400,
    Color   = 0x00004000,
}

impl BitOr for ClearBufferMask {
    type Output = ClearBufferMask;

    fn bitor(self, rhs: ClearBufferMask) -> ClearBufferMask {
        unsafe { mem::transmute(self as u32 | rhs as u32) }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    Points        = 0x0000,
    Lines         = 0x0001,
    LineLoop      = 0x0002,
    LineStrip     = 0x0003,
    Triangles     = 0x0004,
    TriangleStrip = 0x0005,
    TriangleFan   = 0x0006,
    Quads         = 0x0007,
    // GL_QUAD_STRIP
    // GL_POLYGON
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Never                          = 0x0200,
    Less                           = 0x0201,
    Equal                          = 0x0202,
    LEqual                         = 0x0203,
    Greater                        = 0x0204,
    NotEqual                       = 0x0205,
    GEqual                         = 0x0206,
    Always                         = 0x0207,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestFactor {
    Zero             = 0,
    One              = 1,
    SrcColor         = 0x0300,
    OneMinusSrcColor = 0x0301,
    SrcAlpha         = 0x0302,
    OneMinusSrcAlpha = 0x0303,
    DstAlpha         = 0x0304,
    OneMinusDstAlpha = 0x0305,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFactor {
    Zero             = 0,
    One              = 1,
    SrcColor         = 0x0300,
    OneMinusSrcColor = 0x0301,
    SrcAlpha         = 0x0302,
    OneMinusSrcAlpha = 0x0303,
    DstAlpha         = 0x0304,
    OneMinusDstAlpha = 0x0305,
    DstColor         = 0x0306,
    OneMinusDstColor = 0x0307,
    SrcAlphaSaturate = 0x0308,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSeverity {
    High   = 0x9146,
    Medium = 0x9147,
    Low    = 0x9148,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSource {
    API            = 0x8246,
    WindowSystem   = 0x8247,
    ShaderCompiler = 0x8248,
    ThirdParty     = 0x8249,
    Application    = 0x824A,
    Other          = 0x824B,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugType {
    Error              = 0x824C,
    DeprecatedBehavior = 0x824D,
    UndefinedBehavior  = 0x824E,
    Portability        = 0x824F,
    Performance        = 0x8250,
    Other              = 0x8251,
    Marker             = 0x8268,
    PushGroup          = 0x8269,
    PopGroup           = 0x826A,
}

pub extern "C" fn debug_callback(
    source: DebugSource,
    message_type: DebugType,
    _id: UInt,
    severity: DebugSeverity,
    _length: SizeI,
    message: *const u8,
    _user_param: *mut ()) {
    println!(
        "recieved some kind of debug message. source: {:?}, type: {:?}, severity: {:?}, message: {:?}",
        source,
        message_type,
        severity,
        unsafe { CStr::from_ptr(message as *const _) })
}
