extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;

use bootstrap::window::*;
use gl::*;

static VERTEX_POSITIONS: [f32; 9] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     0.0,  1.0, 0.0,
];

fn main() {
    let mut window = Window::new("Hello, Triangle!");

    gl::init();
    let mut vertex_buffer = VertexBuffer::new();
    vertex_buffer.set_data_f32(&VERTEX_POSITIONS[..]);
    vertex_buffer.set_attrib_f32(
        "position",
        AttribLayout {
            elements: 3,
            offset: 0,
            stride: 0,
        });

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        gl::clear();
        DrawBuilder::new(&vertex_buffer, DrawMode::Triangles)
            .map_attrib_location("position", AttributeLocation::from_index(0))
            .draw();
        gl::swap_buffers();
    }
}
