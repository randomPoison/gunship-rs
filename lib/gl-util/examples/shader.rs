extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;
extern crate parse_obj;

use bootstrap::window::*;
use gl::*;
use parse_obj::*;

static VERT_SOURCE: &'static str = r#"
#version 330 core

uniform mat4 model_transform;

in vec4 position;
in vec3 normal_in;
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
    let mut window = Window::new("gl-util - wireframe example");
    gl::init();

    // Compile and link shaders into a shader program.
    let vert_shader = Shader::new(VERT_SOURCE, ShaderType::Vertex).unwrap();
    let frag_shader = Shader::new(FRAG_SOURCE, ShaderType::Fragment).unwrap();
    let program = Program::new(&[vert_shader, frag_shader]).unwrap();

    // Create the vertex buffer and set the vertex attribs.
    let mut vertex_buffer = VertexBuffer::new();
    vertex_buffer.set_data_f32(&*vertex_data);
    vertex_buffer.set_attrib_f32(
        "position",
        AttribLayout {
            elements: 4,
            stride: 7,
            offset: 0,
        });
    vertex_buffer.set_attrib_f32(
        "normal",
        AttribLayout {
            elements: 3,
            stride: 7,
            offset: 4,
        });

    // Create the index buffer.
    let mut index_buffer = IndexBuffer::new();
    index_buffer.set_data_u32(&*indices);

    let mut draw_builder = DrawBuilder::new(&vertex_buffer, DrawMode::Triangles);
    draw_builder
        .index_buffer(&index_buffer)
        .program(&program)
        .map_attrib_name("position", "position")
        .map_attrib_name("normal", "normal_in")
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

        gl::clear();
        draw_builder.draw();
        gl::swap_buffers();
    }
}
