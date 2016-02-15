extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;

use bootstrap::window::*;
use gl::*;

static VERTEX_POSITIONS: [f32; 9] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     0.0,  1.0, 0.0,
];

static VERTEX_INDICES: [u32; 6] = [
    0, 1,
    1, 2,
    2, 0,
];

fn main() {
    let mut window = Window::new("gl-util - wireframe example");

    gl::init();
    let mut vertex_buffer = VertexBuffer::new();
    vertex_buffer.set_data_f32(&VERTEX_POSITIONS[..]);
    vertex_buffer.set_attrib_f32(AttributeLocation::from_index(0), 3, 0, 0);

    let mut index_buffer = IndexBuffer::new();
    index_buffer.set_data_u32(&VERTEX_INDICES[..]);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }

            gl::clear();
            vertex_buffer.draw_elements(DrawMode::Lines, &index_buffer);
            gl::swap_buffers();
        }
    }
}
