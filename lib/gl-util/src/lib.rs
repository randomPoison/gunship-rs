//! Utility wrappers to simplify writing OpenGL code.
//!
//! This crate aspires to provide an abstraction over OpenGL's raw API in order to simplify the
//! task of writing higher-level rendering code for OpenGL. `gl-util` is much in the vein of
//! [glutin](https://github.com/tomaka/glium) and [gfx-rs](https://github.com/gfx-rs/gfx) before
//! it, the main difference being that it is much more poorly constructed and is being developed by
//! someone much less experienced with OpenGL.

extern crate bootstrap_gl as gl;

use gl::{
    BufferName, BufferTarget, BufferUsage, ClearBufferMask, Face, GlType, IndexType, PolygonMode,
    VertexArrayName
};
use std::mem;

pub use gl::{AttributeLocation, DrawMode};
pub use gl::platform::swap_buffers;

/// Initializes global OpenGL state and creates the OpenGL context needed to perform rendering.
pub fn init() {
    gl::create_context();
}

/// TODO: Take clear mask (and values) as parameters.
pub fn clear() {
    unsafe { gl::clear(ClearBufferMask::Color); }
}

/// Represents a buffer of vertex data and the layout of that data.
///
/// Wraps a vertex buffer object and vertex array object into one struct.
#[derive(Debug)]
pub struct VertexBuffer {
    buffer_name: BufferName,
    vertex_array_name: VertexArrayName,
    len: usize,
    element_len: usize,
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
            len: 0,
            element_len: 0,
        }
    }

    /// Fills the buffer with the contents of the data slice.
    pub fn set_data_f32(&mut self, data: &[f32]) {
        self.len = data.len();

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

    /// Specifies how the data for a particular vertex attribute is laid out in the buffer.
    pub fn set_attrib_f32(
        &mut self,
        attrib: AttributeLocation,
        elements: usize,
        stride: usize,
        offset: usize
    ) {
        // Calculate the number of elements based on the attribute.
        if stride == 0 {
            self.element_len = (self.len - offset) / elements;
        } else {
            unimplemented!();
        }

        unsafe {
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);
            gl::bind_vertex_array(self.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib);
            gl::vertex_attrib_pointer(
                attrib,
                elements as i32,
                GlType::Float,
                false,
                stride as i32,
                offset);

            gl::bind_vertex_array(VertexArrayName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
        }
    }

    /// Draws the contents of the vertex buffer to the screen.
    pub fn draw(&self, draw_mode: DrawMode) {
        unsafe {
            gl::bind_vertex_array(self.vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);

            gl::draw_arrays(draw_mode, 0, self.element_len as i32);

            gl::bind_buffer(BufferTarget::Array, BufferName::null());
            gl::bind_vertex_array(VertexArrayName::null());
        }
    }

    /// Draws the contents of the buffer using the specified indices.
    pub fn draw_elements(&self, draw_mode: DrawMode, indices: &IndexBuffer) {
        unsafe {
            gl::bind_vertex_array(self.vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);
            gl::bind_buffer(BufferTarget::ElementArray, indices.buffer_name);

            gl::draw_elements(draw_mode, indices.len as i32, IndexType::UnsignedInt, 0);

            gl::bind_buffer(BufferTarget::ElementArray, BufferName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
            gl::bind_vertex_array(VertexArrayName::null());
        }
    }

    /// Draws the contents of the buffer using the specified indices.
    pub fn draw_wireframe(&self, draw_mode: DrawMode, indices: &IndexBuffer) {
        unsafe {
            gl::bind_vertex_array(self.vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);
            gl::bind_buffer(BufferTarget::ElementArray, indices.buffer_name);
            gl::polygon_mode(Face::FrontAndBack, PolygonMode::Line);

            gl::draw_elements(draw_mode, indices.len as i32, IndexType::UnsignedInt, 0);

            gl::polygon_mode(Face::FrontAndBack, PolygonMode::Fill);
            gl::bind_buffer(BufferTarget::ElementArray, BufferName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
            gl::bind_vertex_array(VertexArrayName::null());
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

#[derive(Debug)]
pub struct IndexBuffer {
    buffer_name: BufferName,
    len: usize,
}

impl IndexBuffer {
    // Create a new index buffer.
    pub fn new() -> IndexBuffer {
        let mut buffer_name = BufferName::null();
        unsafe {
            gl::gen_buffers(1, &mut buffer_name);
        }

        IndexBuffer {
            buffer_name: buffer_name,
            len: 0,
        }
    }

    pub fn set_data_u32(&mut self, data: &[u32]) {
        self.len = data.len();

        let data_ptr = data.as_ptr() as *const ();
        let byte_count = data.len() * mem::size_of::<u32>();

        unsafe {
            gl::bind_buffer(BufferTarget::ElementArray, self.buffer_name);
            gl::buffer_data(
                BufferTarget::ElementArray,
                byte_count as isize,
                data_ptr,
                BufferUsage::StaticDraw);
            gl::bind_buffer(BufferTarget::ElementArray, BufferName::null());
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::delete_buffers(1, &mut self.buffer_name);
        }
    }
}
