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
use std::ops::Deref;

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
pub enum ClearBufferMask {
    Depth         = 0x00000100,
    StencilBuffer = 0x00000400,
    ColorBuffer   = 0x00004000,
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
}

impl Deref for Context {
    type Target = Loader;

    fn deref<'a>(&'a self) -> &'a Loader {
        &self.loader
    }
}

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
                        println!("loading $gl_proc() for the first time.");

                        let $proc_name = (self.proc_loader)(stringify!($gl_proc));
                        self.$proc_name.set(unsafe {
                            mem::transmute($proc_name)
                        });
                    }
                    assert!(self.$proc_name.get().is_some());

                    println!(concat!("calling ", stringify!($gl_proc), "()"));
                    (self.$proc_name.get().unwrap())($( $arg ),*)
                }
            )*
        }
    }
}

gen_proc_loader! {
    glEnable:
        fn enable(capability: ServerCapability),
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
    length: SizeI,
    message: *const u8,
    _user_param: *mut ()) {
    println!(
        "recieved some kind of debug message. source: {:?}, type: {:?}, severity: {:?}, message: {:?}",
        source,
        message_type,
        severity,
        str::from_utf8(unsafe {
            slice::from_raw_parts(message, length as usize)
        }).unwrap());
}
