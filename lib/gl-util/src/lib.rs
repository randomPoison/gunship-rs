//! Utility wrappers to simplify writing OpenGL code.
//!
//! This crate aspires to provide an abstraction over OpenGL's raw API in order to simplify the
//! task of writing higher-level rendering code for OpenGL. `gl-util` is much in the vein of
//! [glutin](https://github.com/tomaka/glium) and [gfx-rs](https://github.com/gfx-rs/gfx) before
//! it, the main difference being that it is much more poorly constructed and is being developed by
//! someone much less experienced with OpenGL.

extern crate bootstrap_gl as gl;

use gl::{BufferName, BufferTarget, BufferUsage, ClearBufferMask, DrawMode, GlType, VertexArrayName};
use std::mem;

pub use gl::{AttributeLocation};
pub use gl::platform::swap_buffers;

/// Initializes global OpenGL state and creates the OpenGL context needed to perform rendering.
pub fn init() {
    gl::create_context();
}

/// Represents a buffer of vertex data and the layout of that data.
///
/// Wraps a vertex buffer object and vertex array object into one struct.
#[derive(Debug)]
pub struct VertexBuffer {
    buffer_name: BufferName,
    vertex_array_name: VertexArrayName,
}

impl VertexBuffer {
    /// Creates a new `VertexBuffer` object.
    pub fn new() -> VertexBuffer {
        let mut buffer_name = BufferName::null();
        let mut vertex_array_name = VertexArrayName::null();
        unsafe {
            gl::gen_buffers(1, &mut buffer_name);
            gl::gen_vertex_arrays(1, &mut vertex_array_name);
        }

        VertexBuffer {
            buffer_name: buffer_name,
            vertex_array_name: vertex_array_name,
        }
    }

    /// Fills the buffer with the contents of the data slice.
    pub fn set_data(&mut self, data: &[f32]) {
        let data_ptr = data.as_ptr() as *const ();
        let byte_count = data.len() * mem::size_of::<f32>();

        unsafe {
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);
            gl::buffer_data(
                BufferTarget::Array,
                byte_count as isize,
                data_ptr,
                BufferUsage::StaticDraw);
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
        }
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::delete_buffers(1, &mut self.buffer_name);
            gl::delete_vertex_arrays(1, &mut self.vertex_array_name);
        }
    }
}
