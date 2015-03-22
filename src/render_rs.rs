#![feature(core, io, old_path)]

extern crate "bootstrap-rs" as bootstrap;
#[macro_use]
extern crate "render_math" as math;
extern crate gl;

use std::io::prelude::*; // TODO: What's up with "prelude" and do I have to manually include it?
use std::fs::File;
use std::f32::consts::PI;

use bootstrap::window::Window;
use bootstrap::window::Message::*;

#[macro_use]
mod geometry;
mod gl_render;

use math::point::Point;
use math::matrix::Matrix4;
use geometry::mesh::Mesh;
use geometry::face::Face;
use gl_render::{GLRender, GLMeshData};

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

    let mesh = gl_test(&renderer);
    let mut mesh_transform = Matrix4::from_rotation(PI * 0.13, 0.0, PI * 0.36); //Matrix4::from_translation(0.5, 0.0, 0.0);
    let frame_rotation = Matrix4::from_rotation(0.0, PI * 0.0001, 0.0);

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
        renderer.draw_mesh(&mesh, mesh_transform);

        if main_window.close {
            break;
        }
    };
}

pub fn load_file(path: &str) -> String {
    let file_path = Path::new(path);
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), why.description()),
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", file_path.display(), why.description()),
        Ok(_) => ()
    }
    contents
}

pub fn gl_test(renderer: &GLRender) -> GLMeshData {

    // create sample mesh data
    let vertex_data: [Point; 9] = [
        point!( 0.0,  0.0,  0.0), // dummy element because obj indices are 1 bases (because obj is dumb).
        point!( 1.0, -1.0, -1.0),
        point!( 1.0, -1.0,  1.0),
        point!(-1.0, -1.0,  1.0),
        point!(-1.0, -1.0, -1.0),
        point!( 1.0,  1.0, -1.0),
        point!( 1.0,  1.0,  1.0),
        point!(-1.0,  1.0,  1.0),
        point!(-1.0,  1.0, -1.0)
    ];

    let face_data: [Face; 12] = [
        face!(4, 2, 1),
        face!(6, 8, 5),
        face!(2, 5, 1),
        face!(3, 6, 2),
        face!(4, 7, 3),
        face!(8, 1, 5),
        face!(4, 3, 2),
        face!(6, 7, 8),
        face!(2, 6, 5),
        face!(3, 7, 6),
        face!(4, 8, 7),
        face!(8, 4, 1)
    ];

    let mesh = Mesh::from_slice(&vertex_data, &face_data);

    let frag_src = load_file("shaders/test3D.frag.glsl");
    let vert_src = load_file("shaders/test3D.vert.glsl");

    renderer.gen_mesh(&mesh,
                      vert_src.as_slice(),
                      frag_src.as_slice())
}
