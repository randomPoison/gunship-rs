extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;

use bootstrap::window::*;
use gl::*;
use gl::context::Context;

static VERTEX_POSITIONS: [f32; 9] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     0.0,  1.0, 0.0,
];

fn main() {
    // Open a window to be used as a target for rendering.
    let mut window = Window::new("Hello, Triangle!");

    // Create the OpenGL context. `Context::new()` will attempt to find a default render target,
    // in this case it will use the window we just opened.
    let context = Context::new().unwrap();

    // Create a vertex buffer to store the vertices of the triangle. We provide it with data and
    // specify the layout of that data.
    let mut vertex_buffer = VertexBuffer::new(&context);
    vertex_buffer.set_data_f32(&VERTEX_POSITIONS[..]);
    vertex_buffer.set_attrib_f32(
        "position",
        AttribLayout {
            elements: 3,
            offset: 0,
            stride: 0,
        });

    // `DrawBuilder` is used to specify all of the various configuration options when drawing. In
    // this case we're using `vertex_buffer` in triangles mode, and we're sending its "position"
    // attribute to attribute location 0, which is the default for `glPosition`.
    let mut draw_builder = DrawBuilder::new(&context, &vertex_buffer, DrawMode::Triangles);
    draw_builder.map_attrib_location("position", AttributeLocation::from_index(0));

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // We use the context to clear the render target and swap buffers. `DrawBuilder` can be
        // used multiple to avoid having to re-configure the build for each draw.
        context.clear();
        draw_builder.draw();
        context.swap_buffers();
    }
}
