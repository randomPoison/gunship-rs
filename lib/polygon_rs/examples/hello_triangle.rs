extern crate bootstrap_rs as bootstrap;
extern crate polygon;

use bootstrap::window::*;
use polygon::camera::*;
use polygon::gl::*;
use polygon::light::*;
use polygon::material::*;
use polygon::math::*;
use polygon::geometry::mesh::*;

static VERTEX_POSITIONS: [f32; 12] = [
    -1.0, -1.0, 0.0, 1.0,
     1.0, -1.0, 0.0, 1.0,
     0.0,  1.0, 0.0, 1.0,
];

static INDICES: [u32; 3] = [0, 1, 2];

fn main() {
    // Open a window and create the renderer instance.
    let mut window = Window::new("Hello, Triangle!");
    let mut renderer = GlRender::new();

    // Build a triangle mesh.
    let mesh = MeshBuilder::new()
        .set_position_data(Point::slice_from_f32_slice(&VERTEX_POSITIONS))
        .set_indices(&INDICES)
        .build()
        .unwrap();

    // Send the mesh to the GPU.
    let gpu_mesh = renderer.gen_mesh(&mesh);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        // Render the mesh.
        renderer.clear();
        renderer.draw_mesh(
            &gpu_mesh,
            &Material::default(),
            Matrix4::identity(),
            Matrix4::identity(),
            &Camera::default(),
            &mut None.into_iter() as &mut Iterator<Item=Light>);
        renderer.swap_buffers();
    }
}
