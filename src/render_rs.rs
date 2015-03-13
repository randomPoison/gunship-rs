#![feature(core)]

extern crate "bootstrap-rs" as bootstrap;
extern crate gl;

use bootstrap::window::{Window, Message};
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

pub static VERTEX_SHADER_SRC: &'static str = r#"
#version 150

in vec4 vertexPos;

void main(void)
{
    gl_Position = vertexPos;
}"#;

pub static FRAGMENT_SHADER_SRC: &'static str = r#"
#version 150

out vec4 fragmentColor;

void main(void)
{
    fragmentColor = vec4(1, 0, 0, 1);
}"#;

// TODO move this up into a more appropriate location
pub fn gl_test(renderer: &GLRender) {
    let vertex_data: [Point; 5] =
    [ point!(0.00, 0.00, 0.00),
      point!(0.50, 0.00, 0.00),
      point!(0.50, 0.50, 0.00),
      point!(0.25, 0.75, 0.00),
      point!(0.00, 0.50, 0.00) ];
    let mesh = Mesh::from_slice(&vertex_data);

    let gl_mesh =
        renderer.gen_mesh(&mesh,
                          VERTEX_SHADER_SRC,
                          FRAGMENT_SHADER_SRC);
    renderer.draw_mesh(&gl_mesh);
}
