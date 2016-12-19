use bootstrap::window::Window;
use gl;
use gl::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::ptr;
use std::rc::Rc;

#[derive(Debug)]
pub struct Context {
    raw: gl::Context,
    inner: Rc<RefCell<ContextInner>>,
}

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
                let _guard = ::context::ContextGuard::new(context);

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
                gl::enable(ServerCapability::FramebufferSrgb);
                gl::enable(ServerCapability::Blend);
            }

            let inner = Rc::new(RefCell::new(ContextInner {
                raw: context,

                server_srgb_enabled: true,
                server_cull_enabled: false,
                server_depth_test_enabled: false,
                server_blend_enabled: true,

                bound_vertex_array: None,
                front_polygon_mode: PolygonMode::default(),
                back_polygon_mode: PolygonMode::default(),
                program: None,
                cull_mode: Face::default(),
                winding_order: WindingOrder::default(),
                depth_test: Comparison::Less,
                blend: Default::default(),
            }));

            Ok(Context {
                raw: context,
                inner: inner,
            })
        }
    }

    /// TODO: Take clear mask (and values) as parameters.
    pub fn clear(&self) {
        let _guard = ::context::ContextGuard::new(self.raw);
        unsafe { gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth); }
    }

    pub fn swap_buffers(&self) {
        let _guard = ::context::ContextGuard::new(self.raw);
        unsafe { gl::platform::swap_buffers(self.raw); }
    }

    pub(crate) fn raw(&self) -> gl::Context {
        self.raw
    }

    pub(crate) fn inner(&self) -> Rc<RefCell<ContextInner>> {
        self.inner.clone()
    }
}

#[derive(Debug)]
pub(crate) struct ContextInner {
    raw: gl::Context,

    server_srgb_enabled: bool,
    server_cull_enabled: bool,
    server_depth_test_enabled: bool,
    server_blend_enabled: bool,

    bound_vertex_array: Option<VertexArrayName>,
    front_polygon_mode: PolygonMode,
    back_polygon_mode: PolygonMode,
    program: Option<ProgramObject>,
    cull_mode: Face,
    winding_order: WindingOrder,
    depth_test: Comparison,
    blend: (SourceFactor, DestFactor),
}

impl ContextInner {
    pub(crate) fn raw(&self) -> gl::Context {
        self.raw
    }

    pub(crate) fn bind_vertex_array(&mut self, vertex_array_name: VertexArrayName) {
        if Some(vertex_array_name) != self.bound_vertex_array {
            unsafe { gl::bind_vertex_array(vertex_array_name); }
            self.bound_vertex_array = Some(vertex_array_name);
        }
    }

    pub(crate) fn unbind_vertex_array(&mut self, vertex_array_name: VertexArrayName) {
        if Some(vertex_array_name) == self.bound_vertex_array {
            unsafe { gl::bind_vertex_array(VertexArrayName::null()); }
            self.bound_vertex_array = None;
        }
    }

    pub(crate) fn polygon_mode(&mut self, mode: PolygonMode) {
        if mode != self.front_polygon_mode || mode != self.back_polygon_mode {
            unsafe { gl::polygon_mode(Face::FrontAndBack, mode); }
            self.front_polygon_mode = mode;
            self.back_polygon_mode = mode;
        }
    }

    pub(crate) fn use_program(&mut self, program: Option<ProgramObject>) {
        if program != self.program {
            match program {
                Some(program) => unsafe { gl::use_program(program); },
                None => unsafe { gl::use_program(ProgramObject::null()); },
            }

            self.program = program;
        }
    }

    pub(crate) fn enable_server_cull(&mut self, enabled: bool) {
        if enabled != self.server_cull_enabled {
            match enabled {
                true => unsafe { gl::enable(ServerCapability::CullFace); },
                false => unsafe { gl::disable(ServerCapability::CullFace); },
            }
            self.server_cull_enabled = enabled;
        }
    }

    pub(crate) fn enable_server_depth_test(&mut self, enabled: bool) {
        if enabled != self.server_depth_test_enabled {
            match enabled {
                true => unsafe { gl::enable(ServerCapability::DepthTest); },
                false => unsafe { gl::disable(ServerCapability::DepthTest); },
            }
            self.server_depth_test_enabled = enabled;
        }
    }

    pub(crate) fn cull_mode(&mut self, face: Face) {
        if self.cull_mode != face {
            unsafe { gl::cull_face(face); }
            self.cull_mode = face;
        }
    }

    pub(crate) fn winding_order(&mut self, winding_order: WindingOrder) {
        if self.winding_order != winding_order {
            unsafe { gl::front_face(winding_order); }
            self.winding_order = winding_order;
        }
    }

    pub(crate) fn depth_test(&mut self, comparison: Comparison) {
        if comparison != self.depth_test {
            unsafe { gl::depth_func(comparison); }
            self.depth_test = comparison;
        }
    }

    pub(crate) fn blend(&mut self, source_factor: SourceFactor, dest_factor: DestFactor) {
        if (source_factor, dest_factor) != self.blend {
            unsafe { gl::blend_func(source_factor, dest_factor); }
            self.blend = (source_factor, dest_factor);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            gl::make_current(self.raw);
            gl::debug_message_callback(None, ptr::null_mut());
            gl::destroy_context(self.raw)
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
pub(crate) struct ContextGuard(gl::Context);

impl ContextGuard {
    pub fn new(context: gl::Context) -> ContextGuard {
        let old = unsafe { gl::make_current(context) };
        ContextGuard(old)
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        // TODO: Assert that the context is still current.
        unsafe { gl::make_current(self.0); }
    }
}
