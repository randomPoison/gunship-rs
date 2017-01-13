//! Utility wrappers to simplify writing OpenGL code.
//!
//! This crate aspires to provide an abstraction over OpenGL's raw API in order to simplify the
//! task of writing higher-level rendering code for OpenGL. `gl-util` is much in the vein of
//! [glutin](https://github.com/tomaka/glium) and [gfx-rs](https://github.com/gfx-rs/gfx),
//! the main difference being that it is much more poorly constructed and is being developed by
//! someone much less OpenGL experience.

#![feature(associated_consts)]
#![feature(pub_restricted)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_gl as gl;

use context::{Context, ContextInner};
use gl::*;
use shader::Program;
use std::mem;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use texture::Texture2d;

pub use gl::{
    AttributeLocation,
    Comparison,
    DestFactor,
    DrawMode,
    Face,
    PolygonMode,
    ShaderType,
    SourceFactor,
    WindingOrder,
};

pub mod context;
pub mod shader;
pub mod texture;

#[cfg(target_os="windows")]
#[path="windows\\mod.rs"]
pub mod platform;

/// Describes the layout of vertex data in a `VertexBuffer`.
///
/// See [`VertexArray::map_attrib_location()`][VertexArray::map_attrib_location] for more information.
///
/// [VertexArray::map_attrib_location]: TODO: Figure out link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttribLayout {
    /// The number of primitive elements per vertex.
    ///
    /// For example, a 3D vector with x, y, and z coordinates has 3 elements. This may not be
    /// larger than 4.
    pub elements: usize,

    /// The distance, in elements, between instances of the vertex attribute.
    ///
    /// Generally this will either be the total number of elements in a vertex, indicating that
    /// vertex attribs are interleaved in the buffer, or 0, indicating that attribs are tightly
    /// packed within the buffer.
    pub stride: usize,

    /// The offset, in elements, from the start of the buffer where the attrib first appears.
    pub offset: usize,
}

#[derive(Debug)]
pub struct VertexArray {
    vertex_array_name: VertexArrayName,
    vertex_buffer_name: BufferName,
    index_buffer: Option<IndexBuffer>,

    /// The number of primitive elments in the buffer.
    ///
    /// Does not reflect the number of vertices in the buffer.
    vertex_primitive_len: usize,

    /// The total number of primitive elements per vertex.
    ///
    /// Used to can determine how many vertices are in the buffer.
    elements_per_vertex: usize,

    context: Rc<RefCell<ContextInner>>,
}

impl VertexArray {
    /// Creates a new VAO and vertex buffer, filling the buffer with the provided data.
    // TODO: Is this operation fallible? If so it should return a `Result<T>`.
    pub fn new(context: &Context, vertex_data: &[f32]) -> VertexArray {
        let context_inner = context.inner();

        let (vertex_buffer_name, vertex_array_name) = unsafe {
            let mut context = context_inner.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());

            // Create the VAO and VBO.
            let vertex_array = gl::gen_vertex_array().expect("Failed to create vertex array object");
            let buffer_name = gl::gen_buffer().expect("Failed to create buffer object");

            // Bind the VAO to the context, then bind the buffer to the VAO.
            context.bind_vertex_array(vertex_array);
            gl::bind_buffer(BufferTarget::Array, buffer_name);

            // Fill the VBO with data.
            gl::buffer_data(
                BufferTarget::Array,
                vertex_data,
                BufferUsage::StaticDraw,
            );

            (buffer_name, vertex_array)
        };

        VertexArray {
            vertex_array_name: vertex_array_name,
            vertex_buffer_name: vertex_buffer_name,
            index_buffer: None,

            vertex_primitive_len: vertex_data.len(),
            elements_per_vertex: 0,

            context: context_inner,
        }
    }

    /// Creates a new VAO with the provided vertex and index data.
    pub fn with_index_buffer(context: &Context, vertex_data: &[f32], index_data: &[u32]) -> VertexArray {
        let mut vertex_array = VertexArray::new(context, vertex_data);

        let index_buffer_name = unsafe {
            let context = vertex_array.context.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());

            let buffer_name = gl::gen_buffer().expect("Failed to generate buffer object");
            gl::bind_buffer(BufferTarget::ElementArray, buffer_name);
            gl::buffer_data(
                BufferTarget::ElementArray,
                index_data,
                BufferUsage::StaticDraw,
            );

            buffer_name
        };

        vertex_array.index_buffer = Some(IndexBuffer {
            name: index_buffer_name,
            primitive_len: index_data.len(),
        });

        vertex_array
    }

    /// Declares a vetex attribute within the vertex buffer.
    pub fn set_attrib(
        &mut self,
        attrib_location: AttributeLocation,
        layout: AttribLayout,
    ) {
        assert!(
            layout.elements <= 4,
            "Layout elements must not be more than 4 (was actually {})",
            layout.elements,
        );
        // TODO: Verify validity of layout?
        // TODO: Verify that `attrib_location` is valid? How would we even do that?

        // Update the total number of elements per vertex.
        self.elements_per_vertex += layout.elements;

        unsafe {
            let mut context = self.context.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());
            context.bind_vertex_array(self.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib_location);
            gl::vertex_attrib_pointer(
                attrib_location,
                layout.elements as i32,
                GlType::Float,
                False,
                (layout.stride * mem::size_of::<f32>()) as i32, // TODO: Correctly handle non-f32
                layout.offset * mem::size_of::<f32>(), // attrib data types.
            );
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        let mut context = self.context.borrow_mut();
        let _guard = ::context::ContextGuard::new(context.raw());
        let buffers = &mut [self.vertex_buffer_name, self.index_buffer.clone().map_or(BufferName::null(), |buf| buf.name)];
        unsafe {
            gl::delete_vertex_arrays(1, &mut self.vertex_array_name);
            gl::delete_buffers(2, buffers.as_ptr());
        }
        context.unbind_vertex_array(self.vertex_array_name);
    }
}

/// Represents a buffer of index data used to index into a `VertexBuffer` when drawing.
#[derive(Debug, Clone, Copy)]
struct IndexBuffer {
    name: BufferName,

    /// The number of indices in the index buffer.
    ///
    /// This does not reflect number of primitive shapes described by the index buffer, e.g. an
    /// index length of 3 may only describe a single triangle.
    primitive_len: usize,
}

/// A configuration object for specifying all of the various configurable options for a draw call.
// TODO: Change `DrawBuidler` to cull backfaces by default.
pub struct DrawBuilder<'a> {
    vertex_array: &'a VertexArray,
    draw_mode: DrawMode,
    polygon_mode: Option<PolygonMode>,
    program: Option<&'a Program>,
    cull: Option<Face>,
    depth_test: Option<Comparison>,
    winding_order: WindingOrder,
    blend: (SourceFactor, DestFactor),
    uniforms: HashMap<UniformLocation, UniformValue<'a>>,

    context: Rc<RefCell<ContextInner>>,
}

impl<'a> DrawBuilder<'a> {
    pub fn new(context: &Context, vertex_array: &'a VertexArray, draw_mode: DrawMode) -> DrawBuilder<'a> {
        // TODO: Make sure `vertex_array` comes from the right context.

        DrawBuilder {
            vertex_array: vertex_array,
            draw_mode: draw_mode,
            polygon_mode: None,
            program: None,
            cull: None,
            depth_test: None,
            winding_order: WindingOrder::default(),
            blend: Default::default(),
            uniforms: HashMap::new(),

            context: context.inner(),
        }
    }

    pub fn polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut DrawBuilder<'a> {
        self.polygon_mode = Some(polygon_mode);
        self
    }

    pub fn program(&mut self, program: &'a Program) -> &mut DrawBuilder<'a> {
        assert!(
            self.context.borrow().raw() == program.context,
            "Specified program's context does not match draw builder's context"
        );
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
        self.winding_order = winding_order;
        self
    }

    pub fn blend(
        &mut self,
        source_factor: SourceFactor,
        dest_factor: DestFactor
    ) -> &mut DrawBuilder<'a> {
        self.blend = (source_factor, dest_factor);
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
        let value = value.into();

        let program =
            self.program.expect("Cannot set a uniform without a shader program");

        // TODO: This checking is bad? Or maybe not? I don't remember.
        let uniform_location = match program.get_uniform_location(name) {
            Some(location) => location,
            None => return self,
        };

        // Add uniform to the uniform map.
        self.uniforms.insert(uniform_location, value);

        self
    }

    pub fn draw(&mut self) {
        let mut context = self.context.borrow_mut();
        let _guard = ::context::ContextGuard::new(context.raw());

        context.polygon_mode(self.polygon_mode.unwrap_or_default());
        context.use_program(self.program.map(Program::inner));

        if let Some(face) = self.cull {
            context.enable_server_cull(true);
            context.cull_mode(face);
            context.winding_order(self.winding_order);
        } else {
            context.enable_server_cull(false);
        }

        if let Some(depth_test) = self.depth_test {
            context.enable_server_depth_test(true);
            context.depth_test(depth_test);
        } else {
            context.enable_server_depth_test(false);
        }

        let (source_factor, dest_factor) = self.blend;
        context.blend(source_factor, dest_factor);

        let mut active_texture = 0;
        // Apply uniforms.
        for (&location, uniform) in &self.uniforms {
            self.apply(uniform, location, &mut active_texture);
        }

        unsafe {
            // TODO: Do a better job tracking VAO and VBO state? I don't know how that would be
            // accomplished, but I don't honestly undertand VAOs so maybe I should figure that out
            // first.
            context.bind_vertex_array(self.vertex_array.vertex_array_name);

            if let Some(indices) = self.vertex_array.index_buffer.as_ref() {
                gl::draw_elements(
                    self.draw_mode,
                    indices.primitive_len as i32,
                    IndexType::UnsignedInt,
                    0,
                );
            } else {
                let vertex_len = self.vertex_array.vertex_primitive_len / self.vertex_array.elements_per_vertex;
                gl::draw_arrays(
                    self.draw_mode,
                    0,
                    vertex_len as i32,
                );
            }
        }
    }

    fn apply(&self, uniform: &UniformValue, location: UniformLocation, active_texture: &mut i32) {
        match *uniform {
            UniformValue::F32(value) => unsafe {
                gl::uniform_f32x1(location, value);
            },
            UniformValue::F32x2((x, y)) => unsafe {
                gl::uniform_f32x2(location, x, y);
            },
            UniformValue::F32x3((x, y, z)) => unsafe {
                gl::uniform_f32x3(location, x, y, z);
            },
            UniformValue::F32x4((x, y, z, w)) => unsafe {
                gl::uniform_f32x4(location, x, y, z, w);
            },
            UniformValue::F32x1v(value) => unsafe {
                gl::uniform_f32x1v(location, value.len() as i32, value.as_ptr());
            },
            UniformValue::F32x3v(value) => unsafe {
                gl::uniform_f32x3v(location, value.len() as i32, value.as_ptr() as *const _);
            },
            UniformValue::F32x4v(value) => unsafe {
                gl::uniform_f32x4v(location, value.len() as i32, value.as_ptr() as *const _);
            },
            UniformValue::I32(value) => unsafe {
                gl::uniform_i32x1(location, value);
            },
            UniformValue::I32x1v(value) => unsafe {
                gl::uniform_i32x1v(location, value.len() as i32, value.as_ptr());
            },
            UniformValue::U32(value) => unsafe {
                gl::uniform_u32x1(location, value);
            },
            UniformValue::Matrix(ref matrix) => match matrix.data.len() {
                16 => unsafe {
                    gl::uniform_matrix_f32x4v(
                        location,
                        1,
                        matrix.transpose.into(),
                        matrix.data.as_ptr())
                },
                9 => unsafe {
                    gl::uniform_matrix_f32x3v(
                        location,
                        1,
                        matrix.transpose.into(),
                        matrix.data.as_ptr())
                },
                _ => panic!("Unsupported matrix data length: {}", matrix.data.len()),
            },
            UniformValue::Texture(texture) => {
                unsafe {
                    texture::set_active_texture(*active_texture as u32);
                    gl::bind_texture(TextureBindTarget::Texture2d, texture.inner());
                    gl::uniform_i32x1(location, *active_texture);
                }

                *active_texture += 1;
            }
        }
    }
}

/// Represents a value for a uniform variable in a shader program.
#[derive(Debug)]
pub enum UniformValue<'a> {
    F32(f32),
    F32x2((f32, f32)),
    F32x3((f32, f32, f32)),
    F32x4((f32, f32, f32, f32)),
    F32x1v(&'a [f32]),
    F32x3v(&'a [[f32; 3]]),
    F32x4v(&'a [[f32; 4]]),
    I32(i32),
    I32x1v(&'a [i32]),
    U32(u32),
    Matrix(GlMatrix<'a>),
    Texture(&'a Texture2d),
}

impl<'a> From<f32> for UniformValue<'a> {
    fn from(value: f32) -> UniformValue<'a> {
        UniformValue::F32(value)
    }
}

impl<'a> From<(f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32)) -> UniformValue<'a> {
        UniformValue::F32x2(value)
    }
}

impl<'a> From<(f32, f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32, f32)) -> UniformValue<'a> {
        UniformValue::F32x3(value)
    }
}

impl<'a> From<(f32, f32, f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32, f32, f32)) -> UniformValue<'a> {
        UniformValue::F32x4(value)
    }
}

impl<'a> From<[f32; 1]> for UniformValue<'a> {
    fn from(value: [f32; 1]) -> UniformValue<'a> {
        UniformValue::F32(value[0])
    }
}

impl<'a> From<[f32; 2]> for UniformValue<'a> {
    fn from(value: [f32; 2]) -> UniformValue<'a> {
        UniformValue::F32x2((value[0], value[1]))
    }
}

impl<'a> From<[f32; 3]> for UniformValue<'a> {
    fn from(value: [f32; 3]) -> UniformValue<'a> {
        UniformValue::F32x3((value[0], value[1], value[2]))
    }
}

impl<'a> From<[f32; 4]> for UniformValue<'a> {
    fn from(value: [f32; 4]) -> UniformValue<'a> {
        UniformValue::F32x4((value[0], value[1], value[2], value[3]))
    }
}

impl<'a> From<&'a [f32]> for UniformValue<'a> {
    fn from(value: &'a [f32]) -> UniformValue<'a> {
        UniformValue::F32x1v(value)
    }
}

impl<'a> From<&'a [[f32; 3]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 3]]) -> UniformValue<'a> {
        UniformValue::F32x3v(value)
    }
}

impl<'a> From<&'a [[f32; 4]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 4]]) -> UniformValue<'a> {
        UniformValue::F32x4v(value)
    }
}

impl<'a> From<i32> for UniformValue<'a> {
    fn from(from: i32) -> UniformValue<'a> {
        UniformValue::I32(from)
    }
}

impl<'a> From<&'a [i32]> for UniformValue<'a> {
    fn from(from: &'a [i32]) -> UniformValue<'a> {
        UniformValue::I32x1v(from)
    }
}

impl<'a> From<u32> for UniformValue<'a> {
    fn from(from: u32) -> UniformValue<'a> {
        UniformValue::U32(from)
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

#[derive(Debug, Clone)]
pub struct GlMatrix<'a> {
    pub data: &'a [f32],
    pub transpose: bool,
}
