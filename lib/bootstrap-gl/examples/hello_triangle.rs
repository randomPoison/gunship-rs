extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_gl as gl;
extern crate stopwatch;

use bootstrap::window::*;
use gl::types::*;
use std::time::{Duration, Instant};
use stopwatch::PrettyDuration;

static VERTEX_POSITIONS: &'static [f32] = &[
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     0.0,  1.0, 0.0,
];

fn main() {
    // Open a window to be used as a target for rendering.
    let mut window = Window::new("Hello, Triangle!").unwrap();

    // Create an OpenGL context for the window.
    let device_context = window.platform().device_context();
    let context = unsafe {
        let context = gl::create_context(device_context).unwrap();
        gl::make_current(context);
        context
    };

    // Create a vertex buffer to store the vertices of the triangle. We provide it with data and
    // specify the layout of that data.
    let buffer_name = gl::gen_buffer().unwrap();
    unsafe {
        gl::bind_buffer(BufferTarget::Array, buffer_name);
        gl::buffer_data(BufferTarget::Array, VERTEX_POSITIONS, BufferUsage::StaticDraw);
    }

    // Create the vertex array object to hold the state needed to draw.
    let vertex_array_name = gl::gen_vertex_array().unwrap();
    unsafe {
        gl::bind_vertex_array(vertex_array_name);
        gl::enable_vertex_attrib_array(AttributeLocation::from_index(0));
        gl::vertex_attrib_pointer(
            AttributeLocation::from_index(0),
            3, // components per vertex
            GlType::Float,
            False,
            0, // stride (bytes)
            0, // offset (bytes)
        );
    }

    unsafe { gl::platform::set_swap_interval(0); }

    let mut last_frame_time = Instant::now();

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        unsafe {
            gl::clear(ClearBufferMask::Color | ClearBufferMask::Depth);

            gl::draw_arrays(
                DrawMode::Triangles,
                0, // offset
                3, // elements to draw
            );
        }

        let swap_time = unsafe {
            let timer = Instant::now();

            gl::swap_buffers(context);

            timer.elapsed()
        };

        if last_frame_time.elapsed() > Duration::new(0, 17_000_000) {
            println!("!!! loop time exceeded: {:?}, swap time: {:?}", PrettyDuration(last_frame_time.elapsed()), PrettyDuration(swap_time));
        } else {
            // println!("loop time: {:?}, swap time: {:?}", PrettyDuration(last_frame_time.elapsed()), PrettyDuration(swap_time));
        }

        // while last_frame_time.elapsed() < Duration::new(0, 16_666_666) {}

        last_frame_time = Instant::now();
    }
}
