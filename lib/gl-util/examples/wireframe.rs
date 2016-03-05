extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;
extern crate parse_obj;

use bootstrap::window::*;
use gl::*;
use parse_obj::*;

fn main() {
    let mut window = Window::new("gl-util - wireframe example");

    // Load mesh file and normalize indices for OpenGL.
    let obj = Obj::from_file("examples/epps_head.obj").unwrap();
    let mut raw_indices = Vec::new();
    for face in obj.faces() {
        for index in face {
            raw_indices.push(*index as u32 - 1);
        }
    }

    gl::init();
    let mut vertex_buffer = VertexBuffer::new();
    vertex_buffer.set_data_f32(obj.raw_positions());
    vertex_buffer.set_attrib_f32(AttributeLocation::from_index(0), 4, 0, 0);

    let mut index_buffer = IndexBuffer::new();
    index_buffer.set_data_u32(&*raw_indices);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }

            gl::clear();
            vertex_buffer.draw_wireframe(DrawMode::Triangles, &index_buffer);
            gl::swap_buffers();
        }
    }
}
