extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
#[macro_use]
extern crate polygon_math as math;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::f32::consts::PI;
use std::error::Error;

use bootstrap::window::Window;
use bootstrap::window::Message::*;

use math::point::Point;
use math::vector::{vector3};
use math::matrix::Matrix4;

use polygon::geometry::mesh::Mesh;
use polygon::geometry::face::Face;
use polygon::gl_render;
use polygon::gl_render::{GLRender, GLMeshData};
use polygon::camera::Camera;

use collada::{GeometricElement, ArrayElement, PrimitiveType};

struct MainWindow
{
    close: bool
}

fn main() {
    let mut main_window = MainWindow {
        close: false
    };

    let instance = bootstrap::init();

    let mut window = Window::new("Rust Window", instance);

    let renderer = gl_render::init(&window);

    let mesh = create_test_mesh(&renderer);
    let mut mesh_transform = Matrix4::from_rotation(PI * 0.13, 0.0, PI * 0.36); //Matrix4::from_translation(0.5, 0.0, 0.0);
    let frame_rotation = Matrix4::from_rotation(0.0, PI * 0.0001, 0.0);

    let mut camera = Camera {
        fov: PI / 3.0,
        aspect: 1.0,
        near: 0.001,
        far: 100.0,

        position: point!(5.0, 5.0, 5.0),
        rotation: Matrix4::from_rotation(0.0, 0.0, 0.0)
    };
    camera.look_at(point!(0.0, 0.0, 0.0), vector3(0.0, 1.0, 0.0));

    loop {
        window.handle_messages();
        loop {
            match window.next_message() {
                Some(message) => {
                    match message {
                        Activate => (),
                        Close => main_window.close = true,
                        Destroy => (),
                        Paint => ()
                    }
                },
                None => break
            }
        }

        mesh_transform = frame_rotation * mesh_transform;
        renderer.draw_mesh(&mesh, mesh_transform, &camera);

        if main_window.close {
            break;
        }
    };
}

pub fn load_file(path: &str) -> String {
    let file_path = Path::new(path);
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", file_path.display(), Error::description(&why)),
        Ok(_) => ()
    }
    contents
}

pub fn create_test_mesh(renderer: &GLRender) -> GLMeshData {
    // load data from COLLADA file
    let file_path = Path::new("meshes/cube.dae");
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    let collada_data = match collada::ColladaData::from_file(&mut file) {
        Err(why) => panic!(why),
        Ok(data) => data
    };

    let mesh = match collada_data.library_geometries.geometries[0].data {
        GeometricElement::Mesh(ref mesh) => mesh,
        _ => panic!("What even is this shit?")
    };

    let vertex_data_raw: &[f32] = match mesh.sources[0].array_element {
        ArrayElement::Float(ref float_array)  => {
            float_array.as_ref()
        },
        _ => panic!("Thas some bullshit.")
    };
    assert!(vertex_data_raw.len() > 0);

    let mut vertex_data: Vec<Point> = Vec::new();
    for offset in (0..vertex_data_raw.len() / 3) {
        vertex_data.push(Point::from_slice(&vertex_data_raw[offset * 3..offset * 3 + 3]));
    }
    assert!(vertex_data.len() > 0);

    let triangles = match mesh.primitives[0] {
        PrimitiveType::Triangles(ref triangles) => triangles,
        _ => panic!("This isn't even cool.")
    };

    let stride = triangles.inputs.len();
    let face_data_raw = triangles.primitives.iter().enumerate().filter_map(|(index, &value)| {
            if index % stride == 0 {
                Some(value as u32)
            } else {
                None
            }
        }).collect::<Vec<u32>>();
    assert!(face_data_raw.len() > 0);

    let mut face_data: Vec<Face> = Vec::new();
    for offset in (0..face_data_raw.len() / 3) {
        face_data.push(Face::from_slice(&face_data_raw[offset * 3..offset * 3 + 3]));
    }
    assert!(face_data.len() > 0);

    let mesh = Mesh::from_slice(vertex_data.as_ref(), face_data.as_ref());

    let frag_src = load_file("shaders/test3D.frag.glsl");
    let vert_src = load_file("shaders/test3D.vert.glsl");

    renderer.gen_mesh(&mesh,
                      vert_src.as_ref(),
                      frag_src.as_ref())
}
