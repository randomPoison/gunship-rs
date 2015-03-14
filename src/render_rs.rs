#![feature(core, io, old_path)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use std::io::prelude::*;
use std::fs::File;

use bootstrap::window::Window;
use bootstrap::window::Message::*;

#[macro_use]
mod geometry;
mod gl_render;

use geometry::point::Point;
use geometry::mesh::Mesh;
use geometry::face::Face;
use gl_render::GLRender;

struct MainWindow
{
    close: bool
}

fn main() {
    let mut main_window = MainWindow {
        close: false
    };

    println!("initializing bootstrap");
    let instance = bootstrap::init();

    println!("creating window");
    let mut window = Window::new("Rust Window", instance);

    let renderer = gl_render::init(&window);

    gl_test(&renderer);

    loop {
        window.handle_messages();
        loop {
            match window.next_message() {
                Some(message) => {
                    println!("message: {:?}", message);
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

        // gl_render::draw_mesh();

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
        Ok(_) => print!("{} contains:\n{}", file_path.display(), contents)
    }
    contents
}

pub fn gl_test(renderer: &GLRender) {

    // create sample mesh data
    let vertex_data: [Point; 8] =
    [point!( 1.0,  1.0,  1.0),
     point!( 1.0,  1.0, -1.0),
     point!( 1.0, -1.0,  1.0),
     point!( 1.0, -1.0, -1.0),
     point!(-1.0,  1.0,  1.0),
     point!(-1.0,  1.0, -1.0),
     point!(-1.0, -1.0,  1.0),
     point!(-1.0, -1.0, -1.0)];

    let face_data: [Face; 1] =
    [face!(0, 4, 2)];
    let mesh = Mesh::from_slice(&vertex_data, &face_data);

    let frag_src = load_file("shaders/test3D.frag.glsl");
    let vert_src = load_file("shaders/test3D.vert.glsl");

    let gl_mesh =
        renderer.gen_mesh(&mesh,
                          vert_src.as_slice(),
                          frag_src.as_slice());
    renderer.draw_mesh(&gl_mesh);
}
