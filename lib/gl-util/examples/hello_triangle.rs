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
    let mut window = Window::new("Hello, Triangle!").unwrap();

    // Create the OpenGL context. `Context::new()` will attempt to find a default render target,
    // in this case it will use the window we just opened.
    let mut context = Context::from_window(&window).unwrap();

    // Create the vertex array object, which groups all buffers for a mesh into a single object.
    let mut vertex_array = VertexArray::new(&context, &VERTEX_POSITIONS[..]);
    vertex_array.set_attrib(
        AttributeLocation::from_index(0),
        AttribLayout { elements: 3, offset: 0, stride: 0 },
    );

    // `DrawBuilder` is used to specify all of the various configuration options when drawing. In
    // this case we're using `vertex_buffer` in triangles mode, and we're sending its "position"
    // attribute to attribute location 0, which is the default for `glPosition`.
    let mut draw_builder = DrawBuilder::new(&mut context, &vertex_array, DrawMode::Triangles);

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
