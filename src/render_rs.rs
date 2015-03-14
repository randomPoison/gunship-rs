#![feature(core, io, old_path)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use std::io::prelude::*;
use std::fs::File;

use bootstrap::window::Window;
use bootstrap::window::Message::*;

#[macro_use]
mod point;
mod mesh;
mod gl_render;

use point::Point;
use mesh::Mesh;
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

pub fn gl_test(renderer: &GLRender) {

    // create sample mesh data
    let vertex_data: [Point; 5] =
    [ point!(0.00, 0.00, 0.00),
      point!(0.50, 0.00, 0.00),
      point!(0.50, 0.50, 0.00),
      point!(0.25, 0.75, 0.00),
      point!(0.00, 0.50, 0.00) ];
    let mesh = Mesh::from_slice(&vertex_data);

    // load shaders
    let vert_path = Path::new("shaders/test.vert.glsl");
    let frag_path = Path::new("shaders/test.frag.glsl");

    let mut vert_file = match File::open(&vert_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", vert_path.display(), why.description()),
        Ok(file) => file,
    };
    let mut frag_file = match File::open(&frag_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", frag_path.display(), why.description()),
        Ok(file) => file,
    };

    let mut vert_src = String::new();
    match vert_file.read_to_string(&mut vert_src) {
        Err(why) => panic!("couldn't read {}: {}", vert_path.display(), why.description()),
        Ok(_) => print!("{} contains:\n{}", vert_path.display(), vert_src),
    }

    let mut frag_src = String::new()    ;
    match frag_file.read_to_string(&mut frag_src) {
        Err(why) => panic!("couldn't read {}: {}", frag_path.display(), why.description()),
        Ok(_) => print!("{} contains:\n{}", frag_path.display(), frag_src),
    }

    let gl_mesh =
        renderer.gen_mesh(&mesh,
                          vert_src.as_slice(),
                          frag_src.as_slice());
    renderer.draw_mesh(&gl_mesh);
}
