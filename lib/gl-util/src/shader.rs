use context::Context;
use gl;
use gl::*;
use std::ffi::CString;
use std::mem;

/// Represents a single shader which can be used to create a `Program`.
#[derive(Debug, Clone)]
pub struct Shader {
    shader_object: ShaderObject,
    shader_type: ShaderType,

    context: ::gl::Context,
}

impl Shader {
    pub fn new<T: AsRef<str>>(context: &Context, source: T, shader_type: ShaderType) -> Result<Shader, ShaderError> {
        let context = context.raw();

        let _context = ::context::ContextGuard::new(context);

        // Create the shader object.
        let shader_object = unsafe { gl::create_shader(shader_type) };
        if shader_object.is_null() {
            return Err(ShaderError::CreateShaderError);
        }

        let source = source.as_ref();
        let source_ptr = source.as_ptr();
        let len = source.len() as i32;

        // Set the shader's source and compile it.
        unsafe {
            gl::shader_source(shader_object, 1, &source_ptr, &len);
            gl::compile_shader(shader_object);
        }

        // Handle compilation failure.
        let compile_status = compile_status(shader_object);
        match compile_status {
            ShaderCompileStatus::Success => Ok(Shader {
                shader_object: shader_object,
                shader_type: shader_type,

                context: context,
            }),
            ShaderCompileStatus::Failure => {
                let log = shader_log(shader_object);
                Err(ShaderError::CompileError(log))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let _context = ::context::ContextGuard::new(self.context);
        unsafe { gl::delete_shader(self.shader_object); }
    }
}

#[derive(Debug, Clone)]
pub enum ShaderError {
    /// Indicates that the call to `gl::create_shader()` returned 0 (the null shader object).
    ///
    /// TODO: Add notes on when this might happen and how to handle this error.
    CreateShaderError,

    /// Indicates that an error occurred while compiling the the shader.
    ///
    /// The inner string is the error log retrieved from OpenGL.
    CompileError(String)
}

fn compile_status(shader_object: ShaderObject) -> ShaderCompileStatus {
    let mut result = 0;
    unsafe {
        gl::get_shader_param(shader_object, ShaderParam::CompileStatus, &mut result);
        mem::transmute(result)
    }
}

fn shader_log(shader_object: ShaderObject) -> String {
    // Get the length of the info log.
    let mut info_log_length = 0;
    unsafe {
        gl::get_shader_param(
            shader_object,
            ShaderParam::InfoLogLength,
            &mut info_log_length);
    }

    // Create the string and read the info log.

    if info_log_length > 0 {
        let mut log = Vec::with_capacity(info_log_length as usize);
        let mut length_out = 0;
        unsafe {
            log.set_len(info_log_length as usize - 1);
            gl::get_shader_info_log(
                shader_object,
                info_log_length,
                &mut length_out,
                log.as_ptr() as *mut _);
        }

        assert!(
            length_out == info_log_length - 1,
            "Expected {} chars out, got {}",
            info_log_length,
            length_out);

        let cstring = unsafe { CString::from_vec_unchecked(log) };
        cstring.into_string().unwrap()
    } else {
        String::new()
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ShaderCompileStatus {
    Failure = 0,
    Success = 1,
}

/// Represents a complete shader program which can be used in rendering.
#[derive(Debug)]
pub struct Program {
    program_object: ProgramObject,

    pub(crate) context: ::gl::Context,
}

impl Program {
    /// Creates a program with the provided shaders.
    pub fn new(context: &Context, shaders: &[Shader]) -> Result<Program, ProgramError> {
        let context = context.raw();

        let _guard = ::context::ContextGuard::new(context);

        // Create shader program.
        let program = Program {
            program_object: unsafe { gl::create_program() },

            context: context,
        };
        if program.inner().is_null() {
            return Err(ProgramError::CreateProgramError);
        }

        // Attach each of the shaders to the program.
        for shader in shaders {
            unsafe { gl::attach_shader(program.inner(), shader.shader_object); }
        }

        // Link the program and detach the shaders.
        unsafe { gl::link_program(program.inner()); }

        // Detach the shaders.
        for shader in shaders {
            unsafe { gl::detach_shader(program.inner(), shader.shader_object); }
        }

        // Check for errors.
        let link_status = link_status(program.inner());
        match link_status {
            ProgramLinkStatus::Success => Ok(program),
            ProgramLinkStatus::Failure => {
                let log = program_log(program.inner());
                Err(ProgramError::LinkError(log))
            }
        }
    }

    /// Gets a vertex attribute location from the program.
    pub fn get_attrib(&self, name: &str) -> Option<AttributeLocation> {
        let _guard = ::context::ContextGuard::new(self.context);

        let mut null_terminated = String::from(name);
        null_terminated.push('\0');

        let raw_location = unsafe { gl::get_attrib_location(self.inner(), null_terminated.as_ptr()) };

        // Check for errors.
        if raw_location == -1 {
            None
        } else {
            Some(AttributeLocation::from_index(raw_location as u32))
        }
    }

    pub(crate) fn get_uniform_location(&self, name: &str) -> Option<UniformLocation> {
        let _guard = ::context::ContextGuard::new(self.context);

        let mut null_terminated = String::from(name);
        null_terminated.push('\0');

        let raw_location = unsafe {
            gl::get_uniform_location(self.inner(), null_terminated.as_ptr())
        };

        // Check for errors.
        if raw_location == -1 {
            None
        } else {
            Some(UniformLocation::from_index(raw_location as u32))
        }
    }

    pub(crate) fn inner(&self) -> ProgramObject {
        self.program_object
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        let _guard = ::context::ContextGuard::new(self.context);
        unsafe { gl::delete_program(self.inner()); }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ProgramLinkStatus {
    Failure = 0,
    Success = 1,
}

#[derive(Debug, Clone)]
pub enum ProgramError {
    /// Indicates that an error occurred while creating the program object.
    ///
    /// TODO: Figure out why this would happen and how to address the error.
    CreateProgramError,

    /// Indicates that an error occurred while linking the program.
    ///
    /// Link errors can occur for various reasons, usually relating to undeclared variables or
    /// variables that are declared differently between different shaders in the program. The
    /// wrapped error message will contain information about the source of the error.
    LinkError(String),
}

fn link_status(program_object: ProgramObject) -> ProgramLinkStatus {
    let mut result = 0;
    unsafe {
        gl::get_program_param(program_object, ProgramParam::LinkStatus, &mut result);
        mem::transmute(result)
    }
}

fn program_log(program_object: ProgramObject) -> String {
    // Get the length of the info log.
    let mut info_log_length = 0;
    unsafe {
        gl::get_program_param(
            program_object,
            ProgramParam::InfoLogLength,
            &mut info_log_length);
    }

    // Create the string and read the info log.

    if info_log_length > 0 {
        let mut log = Vec::with_capacity(info_log_length as usize);
        let mut length_out = 0;
        unsafe {
            log.set_len(info_log_length as usize - 1);
            gl::get_program_info_log(
                program_object,
                info_log_length,
                &mut length_out,
                log.as_ptr() as *mut _);
        }

        assert!(
            length_out == info_log_length - 1,
            "Expected {} chars out, got {}",
            info_log_length,
            length_out);

        let cstring = unsafe { CString::from_vec_unchecked(log) };
        cstring.into_string().unwrap()
    } else {
        String::new()
    }
}
