extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;
extern crate parse_obj;

use bootstrap::window::*;
use gl::*;
use gl::context::Context;
use gl::shader::*;
use parse_obj::Obj;

static VERT_SOURCE: &'static str = r#"
#version 330 core

uniform mat4 model_transform;

layout(location = 0) in vec4 position;
layout(location = 1) in vec3 normal_in;

out vec3 normal;

void main() {
    normal = normal_in;
    gl_Position = model_transform * position;
}
"#;

static FRAG_SOURCE: &'static str = r#"
#version 330 core

uniform vec4 surface_color;

in vec3 normal;
out vec4 fragment_color;

void main() {
    fragment_color = surface_color * vec4(normal, 1);
}
"#;

static MODEL_TRANSFORM: [f32; 16] = [
    0.0, 0.0, -1.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0];

fn main() {
    // Load mesh file and normalize indices for OpenGL.
    let obj = Obj::from_file("examples/epps_head.obj").unwrap();

    // Gather vertex data so that OpenGL can use them.
    let mut vertex_data = Vec::new();

    // Iterate over each of the faces in the mesh.
    for (positions, normals) in obj.position_indices().iter().zip(obj.normal_indices().iter()) {
        // Iterate over each of the vertices in the face to combine the position and normal into
        // a single vertex.
        for (position_index, normal_index) in positions.iter().zip(normals.iter()) {
            let position = obj.positions()[*position_index];
            let normal = obj.normals()[*normal_index];

            vertex_data.extend(&[position.0, position.1, position.2, position.3]);
            vertex_data.extend(&[normal.0, normal.1, normal.2]);
        }
    }

    // Create indices list.
    let indices: Vec<u32> = (0..(obj.position_indices().len() * 3) as u32).collect();

    // Create window and initialize OpenGL.
    let mut window = Window::new("gl-util - wireframe example").unwrap();

    let context = Context::from_window(&window).unwrap();

    // Compile and link shaders into a shader program.
    let vert_shader = Shader::new(&context, VERT_SOURCE, ShaderType::Vertex).unwrap();
    let frag_shader = Shader::new(&context, FRAG_SOURCE, ShaderType::Fragment).unwrap();
    let program = Program::new(&context, &[vert_shader, frag_shader]).unwrap();

    let mut vertex_array = VertexArray::with_index_buffer(&context, &*vertex_data, &*indices);
    vertex_array.set_attrib(
        AttributeLocation::from_index(0),
        AttribLayout { elements: 4, stride: 7, offset: 0 },
    );
    vertex_array.set_attrib(
        AttributeLocation::from_index(1),
        AttribLayout { elements: 3, stride: 7, offset: 4 },
    );

    let mut draw_builder = DrawBuilder::new(&context, &vertex_array, DrawMode::Triangles);
    draw_builder
        .program(&program)
        .uniform("model_transform", GlMatrix {
            data: &MODEL_TRANSFORM,
            transpose: false,
        })
        .uniform("surface_color", (1.0, 0.0, 0.0, 1.0))
        .depth_test(Comparison::Less)
        .cull(Face::Back)
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
