extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

mod components;
mod entity;
mod input;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::f32::consts::PI;
use std::error::Error;

use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::input::ScanCode;

use math::point::{Point, point};
use math::vector::{Vector3, vector3};
use math::matrix::Matrix4;

use polygon::geometry::mesh::Mesh;
use polygon::geometry::face::Face;
use polygon::gl_render;
use polygon::gl_render::{GLRender, GLMeshData};
use polygon::camera::Camera;

use collada::{GeometricElement, ArrayElement, PrimitiveType};

use components::transform::{TransformManager, Transform};
use components::camera::CameraManager;
use entity::EntityManager;

fn main() {
    let mut close = false;

    let instance = bootstrap::init();

    let mut window = Window::new("Rust Window", instance);

    let renderer = gl_render::init(&window);

    let mesh = create_test_mesh(&renderer, "meshes/gun_small.dae");
    let mut mesh_transform = Matrix4::identity();
    let frame_rotation = Matrix4::from_rotation(0.0, PI * 0.0001, 0.0);

    let mut entity_manager = EntityManager::new();
    let camera_entity = entity_manager.create();

    let mut transform_manager = TransformManager::new();
    let transform = transform_manager.create(camera_entity);

    let mut camera_manager = CameraManager::new();

    let mut camera = camera_manager.create(
        PI / 3.0,
        1.0,
        0.001,
        100.0);
    camera.position = point(5.0, 0.0, 5.0);
    camera.look_at(point(0.0, 0.0, 0.0), vector3(0.0, 1.0, 0.0));

    let mut forward = false;
    let mut backward = false;
    let mut left = false;
    let mut right = false;

    let mut last_x = 400;
    let mut last_y = 400;

    let mut rotation_x = 0.0;
    let mut rotation_y = 0.0;

    loop {
        window.handle_messages();
        loop {
            match window.next_message() {
                Some(message) => {
                    match message {
                        Activate => (),
                        Close => close = true,
                        Destroy => (),
                        Paint => (),

                        // Handle inputs.
                        KeyDown(ScanCode::W) => forward = true,
                        KeyUp(ScanCode::W) => forward = false,

                        KeyDown(ScanCode::S) => backward = true,
                        KeyUp(ScanCode::S) => backward = false,

                        KeyDown(ScanCode::D) => right = true,
                        KeyUp(ScanCode::D) => right = false,

                        KeyDown(ScanCode::A) => left = true,
                        KeyUp(ScanCode::A) => left = false,

                        MouseMove(x_coord, y_coord) => {
                            let movement_x = last_x - x_coord;
                            let movement_y = last_y - y_coord;

                            // Add mouse movement to total rotation.
                            rotation_x += (movement_y as f32) * PI * 0.001;
                            rotation_y += (movement_x as f32) * PI * 0.001;

                            // Apply a rotation to the camera based on mouse movmeent.
                            camera.rotation =
                                Matrix4::from_rotation(rotation_x,
                                                       rotation_y,
                                                       0.0);

                            // Save mouse coordinates.
                            last_x = x_coord;
                            last_y = y_coord;
                        }
                        _ => ()
                    }
                },
                None => break
            }
        }

        // Calculate the forward and right vectors.
        let forward_dir = -camera.rotation.z_part();
        let right_dir = camera.rotation.x_part();

        // Move camera based on input.
        if forward {
            camera.position = camera.position + forward_dir * 0.01;
        }

        if backward {
            camera.position = camera.position - forward_dir * 0.01;
        }

        if right {
            camera.position = camera.position + right_dir * 0.01;
        }

        if left {
            camera.position = camera.position - right_dir * 0.01
        }

        // mesh_transform = frame_rotation * mesh_transform;
        renderer.draw_mesh(&mesh, mesh_transform, &camera);

        if close {
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

pub fn create_test_mesh(renderer: &GLRender, path_text: &str) -> GLMeshData {
    // load data from COLLADA file
    let file_path = Path::new(path_text);
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
