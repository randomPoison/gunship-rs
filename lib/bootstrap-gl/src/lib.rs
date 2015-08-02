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

pub use platform::{
    Context,
    init, create_context, swap_buffers, proc_loader,
};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boolean {
    False = 0,
    True = 1,
}

pub type Byte = u8;
pub type UByte = u8;
pub type Short = i16;
pub type UShort = u16;
pub type Int = i32;
pub type UInt = u32;
pub type Fixed = i32;
pub type Int64 = i64;
pub type UInt64 = u64;
pub type SizeI = u32;
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

pub struct Loader {
    proc_loader: fn(&str) -> Option<extern "C" fn()>,
    enable_proc: Cell<Option<extern "C" fn(ServerCapability)>>,
    clear_color_proc: Cell<Option<extern "C" fn(f32, f32, f32, f32)>>,
    debug_message_callback_proc: Cell<Option<extern "C" fn(extern "C" fn(DebugSource, DebugType, UInt, DebugSeverity, SizeI, *const u8, *mut ()), *mut ())>>,
    get_error_proc: Cell<Option<extern "C" fn() -> ErrorCode>>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            proc_loader: platform::proc_loader,
            enable_proc: Cell::new(None),
            clear_color_proc: Cell::new(None),
            debug_message_callback_proc: Cell::new(None),
            get_error_proc: Cell::new(None),
        }
    }

    pub fn enable(&self, capability: ServerCapability) {
        if let None = self.enable_proc.get() {
            println!("loading glEnable() for the first time.");

            let enable_proc = (self.proc_loader)("glEnable");
            self.enable_proc.set(unsafe {
                mem::transmute(enable_proc)
            });
        }
        assert!(self.enable_proc.get().is_some());

        println!("calling glEnable({:?})", capability);
        (self.enable_proc.get().unwrap())(capability);
        println!("error code: {:?}", self.get_error());
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        if let None = self.clear_color_proc.get() {
            println!("loading glClearColor() for the first time.");

            let clear_color_proc = (self.proc_loader)("glClearColor");
            self.clear_color_proc.set(unsafe {
                mem::transmute(clear_color_proc)
            });
        }
        assert!(self.clear_color_proc.get().is_some());

        println!("calling glClearColor()");
        (self.clear_color_proc.get().unwrap())(red, green, blue, alpha);
        println!("error code: {:?}", self.get_error());
    }

    pub fn debug_message_callback(
        &self,
        callback: extern "C" fn(DebugSource, DebugType, UInt, DebugSeverity, SizeI, *const u8, *mut ()),
        user_param: *mut ()) {
        if let None = self.debug_message_callback_proc.get() {
            let debug_message_callback_proc = (self.proc_loader)("glDebugMessageCallback");
            self.debug_message_callback_proc.set(unsafe {
                mem::transmute(debug_message_callback_proc)
            });
        }
        assert!(self.debug_message_callback_proc.get().is_some());

        println!("calling glDebugMessageCallback()");
        (self.debug_message_callback_proc.get().unwrap())(callback, user_param);
        println!("error code: {:?}", self.get_error());
    }

    pub fn get_error(&self) -> ErrorCode {
        if let None = self.get_error_proc.get() {
            let get_error_proc = (self.proc_loader)("glGetError");
            self.get_error_proc.set(unsafe {
                mem::transmute(get_error_proc)
            });
        }
        assert!(self.get_error_proc.get().is_some());

        (self.get_error_proc.get().unwrap())()
    }
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
    id: UInt,
    severity: DebugSeverity,
    length: SizeI,
    message: *const u8,
    user_param: *mut ()) {
    println!(
        "recieved some kind of debug message. source: {:?}, type: {:?}, severity: {:?}, message: {:?}",
        source,
        message_type,
        severity,
        str::from_utf8(unsafe {
            slice::from_raw_parts(message, length as usize)
        }).unwrap());
}
