extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;
extern crate parse_bmp;

use bootstrap::window::*;
use gl::*;
use gl::context::Context;
use gl::shader::*;
use gl::texture::*;
use parse_bmp::{
    Bitmap,
    BitmapData,
};

static VERT_SOURCE: &'static str = r#"
#version 330 core

layout(location = 0) in vec4 position;
layout(location = 1) in vec2 uv;

out vec2 frag_uv;

void main() {
    frag_uv = uv;
    gl_Position = position;
}
"#;

static FRAG_SOURCE: &'static str = r#"
#version 330 core

uniform sampler2D sampler;

in vec2 frag_uv;

out vec4 fragment_color;

void main() {
    fragment_color = texture(sampler, frag_uv) + vec4(frag_uv, 0.0, 1.0) * vec4(0.01, 0.01, 0.01, 1.0);
}
"#;

static VERTEX_DATA: &'static [f32] = &[
    -1.0,  3.0, 0.0, // Position
     0.0,  2.0,      // Uv

     3.0, -1.0, 0.0, // Position
     2.0,  0.0,      // Uv

    -1.0, -1.0, 0.0, // Position
     0.0,  0.0,      // Uv
];

static TEXTURE_DATA: &'static [u8] = include_bytes!("./structured.bmp");

fn main() {
    // Create window and initialize OpenGL.
    let mut window = Window::new("gl-util - texture example").unwrap();
    let context = Context::from_window(&window).unwrap();

    // Compile and link shaders into a shader program.
    let vert_shader = Shader::new(&context, VERT_SOURCE, ShaderType::Vertex).unwrap();
    let frag_shader = Shader::new(&context, FRAG_SOURCE, ShaderType::Fragment).unwrap();
    let program = Program::new(&context, &[vert_shader, frag_shader]).unwrap();

    let mut vertex_array = VertexArray::new(&context, &VERTEX_DATA[..]);
    vertex_array.set_attrib(
        AttributeLocation::from_index(0),
        AttribLayout { elements: 3, offset: 0, stride: 5 },
    );
    vertex_array.set_attrib(
        AttributeLocation::from_index(1),
        AttribLayout { elements: 2, offset: 3, stride: 5 },
    );

    // Parse the bitmap and setup the texture.
    let bitmap = Bitmap::from_bytes(TEXTURE_DATA).unwrap();
    let data = match bitmap.data() {
        &BitmapData::Bgr(ref data) => &**data,
        _ => panic!("Nuh uh, that file is a bitmap"),
    };

    let texture = Texture2d::new(
        &context,
        TextureFormat::Bgr,
        TextureInternalFormat::Rgb,
        bitmap.width(),
        bitmap.height(),
        data)
        .unwrap();

    let mut draw_builder = DrawBuilder::new(&context, &vertex_array, DrawMode::Triangles);
    draw_builder
        .program(&program)
        .uniform("sampler", &texture)
        .winding(WindingOrder::Clockwise);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        context.clear();
        draw_builder.draw();
        context.swap_buffers();
    }
}
