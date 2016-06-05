//! Utility wrappers to simplify writing OpenGL code.
//!
//! This crate aspires to provide an abstraction over OpenGL's raw API in order to simplify the
//! task of writing higher-level rendering code for OpenGL. `gl-util` is much in the vein of
//! [glutin](https://github.com/tomaka/glium) and [gfx-rs](https://github.com/gfx-rs/gfx) before
//! it, the main difference being that it is much more poorly constructed and is being developed by
//! someone much less experienced with OpenGL.

extern crate bootstrap_gl as gl;

use gl::{
    BufferName, BufferTarget, BufferUsage, ClearBufferMask, debug_callback, False, GlType,
    IndexType, ProgramObject, ServerCapability, TextureBindTarget, TextureObject, UniformLocation,
    VertexArrayName,
};
use std::{mem, ptr};
use std::cell::Cell;
use std::collections::HashMap;
use texture::Texture2d;

pub use gl::{
    AttributeLocation, Comparison, DestFactor, DrawMode, Face, PolygonMode, ShaderType,
    SourceFactor, WindingOrder,
};
pub use gl::platform::swap_buffers;
pub use self::shader::*;

pub mod shader;
pub mod texture;

/// Initializes global OpenGL state and creates the OpenGL context needed to perform rendering.
pub fn init() {
    gl::create_context();
    unsafe {
        gl::enable(ServerCapability::DebugOutput);
        gl::debug_message_callback(debug_callback, ptr::null_mut());
    }
}

/// TODO: Take clear mask (and values) as parameters.
pub fn clear() {
    unsafe { gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth); }
}

/// Represents a buffer of vertex data and the layout of that data.
///
/// Wraps a vertex buffer object and vertex array object into one struct.
#[derive(Debug)]
pub struct VertexBuffer {
    buffer_name: BufferName,
    len: usize,
    element_len: usize,
    attribs: HashMap<String, AttribLayout>,
}

impl VertexBuffer {
    /// Creates a new `VertexBuffer` object.
    pub fn new() -> VertexBuffer {
        let mut buffer_name = BufferName::null();
        unsafe {
            gl::gen_buffers(1, &mut buffer_name);
        }

        VertexBuffer {
            buffer_name: buffer_name,
            len: 0,
            element_len: 0,
            attribs: HashMap::new(),
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
    ///
    /// # Parameters
    ///
    /// - `attrib` - The attribute being set. This can be gotten using `Program::get_attrib()`.
    /// - `elements` - The number of `f32`s per vertex for this attribute. For example, for a
    ///   vertex normal with x, y, and z coordinates `elements` would be 3.
    /// - `stride` - The offset in elements between consecutive vertex attributes. If `stride` is
    ///   attributes are understood to be tightly packed.
    /// - `offset` - The offset in elements from the start of the buffer where the first attribute
    ///   is located.
    pub fn set_attrib_f32<T: Into<String>>(
        &mut self,
        attrib: T,
        layout: AttribLayout,
    ) {
        // Calculate the number of elements based on the attribute.
        // TODO: Verify that each attrib has the same element length.
        self.element_len = (self.len - layout.offset) / layout.elements + layout.stride;
        self.attribs.insert(attrib.into(), layout);
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::delete_buffers(1, &mut self.buffer_name);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttribLayout {
    pub elements: usize,
    pub stride: usize,
    pub offset: usize,
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

/// A configuration object for specifying all of the various configurable options for a draw call.
pub struct DrawBuilder<'a> {
    vertex_array_name: VertexArrayName,
    vertex_buffer: &'a VertexBuffer,
    draw_mode: DrawMode,
    index_buffer: Option<&'a IndexBuffer>,
    polygon_mode: Option<PolygonMode>,
    program: Option<&'a Program>,
    cull: Option<Face>,
    depth_test: Option<Comparison>,
    winding_order: Option<WindingOrder>,
    blend: Option<(SourceFactor, DestFactor)>,
    uniforms: HashMap<UniformLocation, UniformValue<'a>>,

    active_texture: Cell<i32>,
}

impl<'a> DrawBuilder<'a> {
    pub fn new(vertex_buffer: &'a VertexBuffer, draw_mode: DrawMode) -> DrawBuilder<'a> {
        let mut vertex_array_name = VertexArrayName::null();
        unsafe {
            gl::gen_vertex_arrays(1, &mut vertex_array_name);
        }

        DrawBuilder {
            vertex_array_name: vertex_array_name,
            vertex_buffer: vertex_buffer,
            draw_mode: draw_mode,
            index_buffer: None,
            polygon_mode: None,
            program: None,
            cull: None,
            depth_test: None,
            winding_order: None,
            blend: None,
            uniforms: HashMap::new(),

            active_texture: Cell::new(0),
        }
    }

    pub fn index_buffer(&mut self, index_buffer: &'a IndexBuffer) -> &mut DrawBuilder<'a> {
        self.index_buffer = Some(index_buffer);
        self
    }

    pub fn polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut DrawBuilder<'a> {
        self.polygon_mode = Some(polygon_mode);
        self
    }

    pub fn program(&mut self, program: &'a Program) -> &mut DrawBuilder<'a> {
        self.program = Some(program);
        self
    }

    pub fn cull(&mut self, face: Face) -> &mut DrawBuilder<'a> {
        self.cull = Some(face);
        self
    }

    pub fn depth_test(&mut self, comparison: Comparison) -> &mut DrawBuilder<'a> {
        self.depth_test = Some(comparison);
        self
    }

    pub fn winding(&mut self, winding_order: WindingOrder) -> &mut DrawBuilder<'a> {
        self.winding_order = Some(winding_order);
        self
    }

    pub fn blend(
        &mut self,
        source_factor: SourceFactor,
        dest_factor: DestFactor
    ) -> &mut DrawBuilder<'a> {
        if source_factor == SourceFactor::One && dest_factor == DestFactor::Zero {
            self.blend = None;
        } else {
            self.blend = Some((source_factor, dest_factor));
        }
        self
    }

    /// Maps a vertex attribute to an attribute location for the current program.
    ///
    /// # Panics
    ///
    /// - If the the vertex buffer does not have an attribute named `buffer_attrib_name`.
    pub fn map_attrib_location(
        &mut self,
        buffer_attrib_name: &str,
        attrib_location: AttributeLocation
    ) -> &mut DrawBuilder<'a> {
        let layout = match self.vertex_buffer.attribs.get(buffer_attrib_name) {
            Some(&attrib_data) => attrib_data,
            None => panic!("Vertex buffer has no attribute \"{}\"", buffer_attrib_name),
        };

        unsafe {
            gl::bind_buffer(BufferTarget::Array, self.vertex_buffer.buffer_name);
            gl::bind_vertex_array(self.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib_location);
            gl::vertex_attrib_pointer(
                attrib_location,
                layout.elements as i32,
                GlType::Float,
                False,
                (layout.stride * mem::size_of::<f32>()) as i32,
                layout.offset * mem::size_of::<f32>());

            gl::bind_vertex_array(VertexArrayName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
        }

        self
    }

    /// Maps a vertex attribute to a variable name in the shader program.
    ///
    /// `map_attrib_name()` will silently ignore a program that does not have an input variable
    /// named `program_attrib_name` or a vertex buffer that does not have an attribute named
    /// `buffer_attrib_name`, so it is always safe to speculatively map vertex attributes
    /// even when the shader program may not use that attribute.
    ///
    /// # Panics
    ///
    /// - If the program has not been set using `program()`.
    pub fn map_attrib_name(
        &mut self,
        buffer_attrib_name: &str,
        program_attrib_name: &str
    ) -> &mut DrawBuilder<'a> {
        let program = self.program.expect("Cannot map attribs without a shader program");
        let attrib = match program.get_attrib(program_attrib_name) {
            Some(attrib) => attrib,
            None => return self,
        };
        let layout = match self.vertex_buffer.attribs.get(buffer_attrib_name) {
            Some(&attrib_data) => attrib_data,
            None => return self,
        };

        unsafe {
            gl::bind_buffer(BufferTarget::Array, self.vertex_buffer.buffer_name);
            gl::bind_vertex_array(self.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib);
            gl::vertex_attrib_pointer(
                attrib,
                layout.elements as i32,
                GlType::Float,
                False,
                (layout.stride * mem::size_of::<f32>()) as i32,
                layout.offset * mem::size_of::<f32>());

            gl::bind_vertex_array(VertexArrayName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
        }

        self
    }

    /// Sets the value of a uniform variable in the shader program.
    ///
    /// `uniform()` will silently ignore uniform variables that do not exist in the shader program,
    /// so it is always safe to speculatively set uniform values even if the shader program may
    /// not use that uniform.
    ///
    /// # Panics
    ///
    /// - If the program has not been set using `program()`.
    pub fn uniform<T>(
        &mut self,
        name: &str,
        value: T
    ) -> &mut DrawBuilder<'a>
        where T: Into<UniformValue<'a>>
    {
        let program =
            self.program.expect("Cannot set a uniform without a shader program");
        let uniform_location = match program.get_uniform_location(name) {
            Some(location) => location,
            None => {
                println!("no uniform named {:?}", name);
                return self;
            },
        };

        // Add uniform to the uniform map.
        self.uniforms.insert(uniform_location, value.into());

        self
    }

    pub fn draw(&self) {
        unsafe {
            gl::enable(ServerCapability::FramebufferSrgb);

            gl::bind_vertex_array(self.vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, self.vertex_buffer.buffer_name);

            if let Some(polygon_mode) = self.polygon_mode {
                gl::polygon_mode(Face::FrontAndBack, polygon_mode);
            }

            if let Some(program) = self.program {
                let Program(program_object) = *program;
                gl::use_program(program_object);
            }

            if let Some(face) = self.cull {
                gl::enable(ServerCapability::CullFace);
                gl::cull_face(face);

                if let Some(winding_order) = self.winding_order {
                    gl::front_face(winding_order);
                }
            }

            if let Some(depth_test) = self.depth_test {
                gl::enable(ServerCapability::DepthTest);
                gl::depth_func(depth_test);
            }

            if let Some((source_factor, dest_factor)) = self.blend {
                gl::enable(ServerCapability::Blend);
                gl::blend_func(source_factor, dest_factor);
            }

            // Apply uniforms.
            for (&location, uniform) in &self.uniforms {
                self.apply(uniform, location);
            }

            if let Some(indices) = self.index_buffer {
                gl::bind_buffer(BufferTarget::ElementArray, indices.buffer_name);
                gl::draw_elements(
                    self.draw_mode,
                    indices.len as i32,
                    IndexType::UnsignedInt,
                    0);
            } else {
                gl::draw_arrays(
                    self.draw_mode,
                    0,
                    self.vertex_buffer.element_len as i32);
            }

            // Reset all values even if they weren't used so that we don't need to branch twice on
            // each option.

            // Reset all used textures.
            for texture in 0..self.active_texture.get() {
                texture::set_active_texture(texture as u32);
                gl::bind_texture(TextureBindTarget::Texture2d, TextureObject::null());
            }
            self.active_texture.set(0);

            gl::front_face(WindingOrder::CounterClockwise);
            gl::disable(ServerCapability::Blend);
            gl::disable(ServerCapability::DepthTest);
            gl::disable(ServerCapability::CullFace);
            gl::polygon_mode(Face::FrontAndBack, PolygonMode::Fill);
            gl::use_program(ProgramObject::null());
            gl::bind_buffer(BufferTarget::ElementArray, BufferName::null());
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
            gl::bind_vertex_array(VertexArrayName::null());
        }
    }

    fn apply(&self, uniform: &UniformValue, location: UniformLocation) {
        match *uniform {
            UniformValue::F32x1(value) => unsafe {
                gl::uniform_f32x1(location, value);
            },
            UniformValue::F32x4((x, y, z, w)) => unsafe {
                gl::uniform_f32x4(location, x, y, z, w);
            },
            UniformValue::Matrix(ref matrix) => match matrix.data.len() {
                16 => unsafe {
                    gl::uniform_matrix_f32x4v(
                        location,
                        1,
                        matrix.transpose.into(),
                        matrix.data.as_ptr())
                },
                9 => unimplemented!(),
                _ => panic!("Unsupported matrix data length: {}", matrix.data.len()),
            },
            UniformValue::Texture(texture) => {
                let active_texture = self.active_texture.get();

                unsafe {
                    texture::set_active_texture(active_texture as u32);
                    gl::bind_texture(TextureBindTarget::Texture2d, texture.raw_value());
                    gl::uniform_i32x1(location, active_texture);
                }

                self.active_texture.set(active_texture + 1);
            }
        }
    }
}

impl<'a> Drop for DrawBuilder<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::delete_vertex_arrays(1, &mut self.vertex_array_name);
        }
    }
}

/// Represents a value for a uniform variable in a shader program.
pub enum UniformValue<'a> {
    F32x1(f32),
    F32x4((f32, f32, f32, f32)),
    Matrix(GlMatrix<'a>),
    Texture(&'a Texture2d),
}

impl<'a> From<f32> for UniformValue<'a> {
    fn from(value: f32) -> UniformValue<'a> {
        UniformValue::F32x1(value)
    }
}

impl<'a> From<(f32, f32, f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32, f32, f32)) -> UniformValue<'a> {
        UniformValue::F32x4(value)
    }
}

impl<'a> From<[f32; 4]> for UniformValue<'a> {
    fn from(value: [f32; 4]) -> UniformValue<'a> {
        UniformValue::F32x4((value[0], value[1], value[2], value[3]))
    }
}

impl<'a> From<GlMatrix<'a>> for UniformValue<'a> {
    fn from(matrix: GlMatrix<'a>) -> UniformValue<'a> {
        UniformValue::Matrix(matrix)
    }
}

impl<'a> From<&'a Texture2d> for UniformValue<'a> {
    fn from(from: &'a Texture2d) -> UniformValue<'a> {
        UniformValue::Texture(from)
    }
}

pub struct GlMatrix<'a> {
    pub data: &'a [f32],
    pub transpose: bool,
}

/// Represents a complete shader program which can be used in rendering.
#[derive(Debug)]
pub struct Program(ProgramObject);

impl Program {
    fn get_uniform_location(&self, name: &str) -> Option<UniformLocation> {
        let Program(program_object) = *self;

        let mut null_terminated = String::from(name);
        null_terminated.push('\0');

        let raw_location = unsafe {
            gl::get_uniform_location(program_object, null_terminated.as_ptr())
        };

        // Check for errors.
        if raw_location == -1 {
            None
        } else {
            Some(UniformLocation::from_index(raw_location as u32))
        }
    }
}
