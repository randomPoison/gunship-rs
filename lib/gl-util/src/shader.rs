use gl;
use gl::*;
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct Shader(ShaderObject);

impl Shader {
    pub fn new(source: &str, shader_type: ShaderType) -> Result<Shader, ShaderError> {
        // Create the shader object.
        let shader_object = unsafe { gl::create_shader(shader_type) };
        if shader_object.is_null() {
            return Err(ShaderError::CreateShaderError);
        }

        let source_ptr = source.as_ptr();
        let len = source.len() as i32;

        // Set the shader's source and compile it.
        unsafe {
            gl::shader_source(shader_object, 1, &source_ptr, &len);
            gl::compile_shader(shader_object);
        }

        // Handle compilation failure.
        let compile_status = compile_status(shader_object);
        if compile_status == ShaderCompileStatus::Failure {
            let log = read_log(shader_object);
            return Err(ShaderError::CompileError(log));
        }

        Ok(Shader(shader_object))
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
    }

    if result == 1 {
        ShaderCompileStatus::Success
    } else if result == 0 {
        ShaderCompileStatus::Failure
    } else {
        panic!(
            "gl::get_shader_param(CompileStatus) returned a value other than 0 or 1: {}",
            result);
    }
}

fn read_log(shader_object: ShaderObject) -> String {
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
enum ShaderCompileStatus {
    Success = 1,
    Failure = 0,
}
