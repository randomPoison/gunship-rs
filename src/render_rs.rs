#![feature(core, io, old_path)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use std::io::prelude::*;
use std::fs::File;

use bootstrap::window::Window;
use bootstrap::window::Message::*;

#[macro_use]
mod geometry;
#[macro_use]
mod math;
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
    let mesh_transform = Matrix4::from_translation(0.5, 0.0, 0.0);

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
        face!(1, 2, 4),
        face!(5, 8, 6),
        face!(1, 5, 2),
        face!(2, 6, 3),
        face!(3, 7, 4),
        face!(5, 1, 8),
        face!(2, 3, 4),
        face!(8, 7, 6),
        face!(5, 6, 2),
        face!(6, 7, 3),
        face!(7, 8, 4),
        face!(1, 4, 8)
    ];

    let mesh = Mesh::from_slice(&vertex_data, &face_data);

    let frag_src = load_file("shaders/test3D.frag.glsl");
    let vert_src = load_file("shaders/test3D.vert.glsl");

    renderer.gen_mesh(&mesh,
                      vert_src.as_slice(),
                      frag_src.as_slice())
}
