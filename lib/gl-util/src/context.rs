use gl::{
    self,
    ClearBufferMask,
    DebugSeverity,
    DebugSource,
    DebugType,
    ServerCapability,
    StringName,
};
use std::ffi::CStr;
use std::ptr;

#[derive(Debug)]
pub struct Context {
    device_context: gl::DeviceContext,
    render_context: gl::Context,
}

impl Context {
    /// Creates a new rendering context.
    ///
    /// Attempts to find a currently active window and use that as the target for rendering. If
    /// no window can be found then `Err` will be returned.
    pub fn new() -> Result<Context, Error> {
        if let Some(device_context) = ::platform::find_device_context() {
            Context::from_device_context(device_context)
        } else {
            Err(Error::NoDeviceContext)
        }
    }

    /// Initializes global OpenGL state and creates the OpenGL context needed to perform rendering.
    fn from_device_context(device_context: gl::DeviceContext) -> Result<Context, Error> {
        pub extern "system" fn debug_callback(
            source: DebugSource,
            message_type: DebugType,
            object_id: u32,
            severity: DebugSeverity,
            _length: i32,
            message: *const u8,
            _user_param: *mut ()
        ) {
            use std::ffi::CStr;

            let message = unsafe { CStr::from_ptr(message as *const _) }.to_string_lossy();

            println!(
                r#"Recieved some kind of debug message.
                source: {:?},
                type: {:?},
                object_id: 0x{:x},
                severity: {:?},
                message: {}"#,
                source,
                message_type,
                object_id,
                severity,
                message);
        }

        unsafe {
            gl::init(device_context);

            let context =
                gl::create_context(device_context)
                .ok_or(Error::UnableToCreateRenderContext)?;

            gl::enable(ServerCapability::DebugOutput);
            gl::debug_message_callback(Some(debug_callback), ptr::null_mut());

            let vendor = CStr::from_ptr(gl::get_string(StringName::Vendor)).to_str().unwrap();
            let renderer = CStr::from_ptr(gl::get_string(StringName::Renderer)).to_str().unwrap();
            let version = CStr::from_ptr(gl::get_string(StringName::Version)).to_str().unwrap();
            let glsl_version = CStr::from_ptr(gl::get_string(StringName::ShadingLanguageVersion)).to_str().unwrap();

            println!("OpenGL Information:");
            println!("\tvendor: {}", vendor);
            println!("\trenderer: {}", renderer);
            println!("\tversion: {}", version);
            println!("\tglsl version: {}", glsl_version);

            Ok(Context {
                device_context: device_context,
                render_context: context,
            })
        }
    }

    /// TODO: Take clear mask (and values) as parameters.
    pub fn clear(&self) {
        unsafe { gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth); }
    }

    pub fn swap_buffers(&self) {
        unsafe { gl::platform::swap_buffers(self.device_context); }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            gl::make_current(self.device_context, self.render_context);
            gl::debug_message_callback(None, ptr::null_mut());
            gl::destroy_context(self.render_context);
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// Indicates that the program was unable to find an active device context.
    ///
    /// This error can occur if no compatible window is active, or if the program was unable to
    /// find the device context for the window.
    NoDeviceContext,

    /// Indicates that the program failed the create the rendering context.
    ///
    /// This might happen because reasons.
    UnableToCreateRenderContext,
}
