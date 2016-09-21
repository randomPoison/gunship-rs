use bootstrap::window::Window;
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
use std::marker::PhantomData;
use std::ptr;

#[derive(Debug)]
pub struct Context(gl::Context);

impl Context {
    /// Creates a new rendering context for the specified window.
    pub fn from_window(window: &Window) -> Result<Context, Error> {
        let device_context = window.platform().device_context();
        Context::from_device_context(device_context)
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
            let context =
                gl::create_context(device_context)
                .ok_or(Error::UnableToCreateRenderContext)?;

            {
                let _guard = ::context::ContextGuard::new(&context);

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

                // Load a bunch of proc pointers for funsies.
                gl::get_attrib_location::load();
                gl::gen_vertex_arrays::load();
            }

            Ok(Context(context))
        }
    }

    /// TODO: Take clear mask (and values) as parameters.
    pub fn clear(&self) {
        let _guard = ::context::ContextGuard::new(&self.0);
        unsafe { gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth); }
    }

    pub fn swap_buffers(&self) {
        let _guard = ::context::ContextGuard::new(&self.0);
        unsafe { gl::platform::swap_buffers(self.0); }
    }

    /// Returns the underlying opengl context.
    ///
    /// This is used internally in conjunction with `ContextGuard` to ensure that any use of the
    /// OpenGL api is correctly done with the correct context and that objects from different
    /// contexts are not used together.
    pub(crate) fn inner(&self) -> gl::Context {
        self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            gl::make_current(self.0);
            gl::debug_message_callback(None, ptr::null_mut());
            gl::destroy_context(self.0)
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

#[derive(Debug)]
pub(crate) struct ContextGuard<'a> {
    old: gl::Context,
    _phantom: PhantomData<&'a gl::Context>,
}

impl<'a> ContextGuard<'a> {
    pub fn new(context: &gl::Context) -> ContextGuard {
        let old = unsafe { gl::make_current(*context) };
        ContextGuard {
            old: old,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Drop for ContextGuard<'a> {
    fn drop(&mut self) {
        // TODO: Assert that the context is still current.
        unsafe { gl::make_current(self.old); }
    }
}
