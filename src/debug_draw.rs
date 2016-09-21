use std::ptr;
use std::sync::Mutex;

use math::*;
use polygon::{Renderer, GpuMesh};
use polygon::geometry::mesh::MeshBuilder;

static mut instance: *const Mutex<DebugDrawInner> = 0 as *const _;

type MeshIndex = u32;

static CUBE_VERTS: [f32; 32] =
    [ 0.5,  0.5,  0.5, 1.0,
      0.5,  0.5, -0.5, 1.0,
      0.5, -0.5,  0.5, 1.0,
      0.5, -0.5, -0.5, 1.0,
     -0.5,  0.5,  0.5, 1.0,
     -0.5,  0.5, -0.5, 1.0,
     -0.5, -0.5,  0.5, 1.0,
     -0.5, -0.5, -0.5, 1.0,];

static CUBE_INDICES: [MeshIndex; 24] =
    [0, 1,
     1, 3,
     3, 2,
     2, 0,
     4, 5,
     5, 7,
     7, 6,
     6, 4,
     0, 4,
     1, 5,
     2, 6,
     3, 7,];

pub struct DebugDraw {
    // material: Material,
    _unit_cube: GpuMesh,
    _unit_sphere: GpuMesh,

    inner: Box<Mutex<DebugDrawInner>>,

    // Vecs used for dynamically reconstructing meshes.
    _line_vertices: Vec<f32>,
    _line_indices: Vec<MeshIndex>,
}

impl DebugDraw {
    pub fn new(renderer: &mut Renderer) -> DebugDraw {
        assert!(unsafe { instance.is_null() }, "Cannot create more than one instance of DebugDraw at a time");

        let mut inner = Box::new(Mutex::new(DebugDrawInner {
            command_buffer: Vec::new(),
        }));

        unsafe {
            instance = &mut *inner;
        }

        // Build sphere vertices. This could be done offline, but I'm lazy so we do it at runtime!
        let unit_sphere = {
            const VERTS_PER_CIRCLE: usize = 50;
            let mut sphere_verts = Vec::new();
            let mut sphere_indices = Vec::<MeshIndex>::new();

            // Vertices around X axis.
            sphere_verts.push(Point::new(0.0, 0.0, 1.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(0.0, theta.sin(), theta.cos()));
                sphere_indices.push(sphere_verts.len() as MeshIndex - 2);
                sphere_indices.push(sphere_verts.len() as MeshIndex - 1);
            }

            // Vertices around Y axis.
            sphere_verts.push(Point::new(1.0, 0.0, 0.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(theta.cos(), 0.0, theta.sin()));
                sphere_indices.push(sphere_verts.len() as MeshIndex - 2);
                sphere_indices.push(sphere_verts.len() as MeshIndex - 1);
            }

            // Vertices around Z axis.
            sphere_verts.push(Point::new(1.0, 0.0, 0.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(theta.cos(), theta.sin(), 0.0));
                sphere_indices.push(sphere_verts.len() as MeshIndex - 2);
                sphere_indices.push(sphere_verts.len() as MeshIndex - 1);
            }

            build_mesh(renderer, Point::as_ref(&*sphere_verts), &*sphere_indices)
        };

        DebugDraw {
            // material: resource_manager.get_material("lib/polygon_rs/resources/materials/diffuse_flat.material").unwrap().clone(),
            _unit_cube: build_mesh(&mut *renderer, &CUBE_VERTS, &CUBE_INDICES),
            _unit_sphere: unit_sphere,

            inner: inner,

            _line_vertices: Vec::new(),
            _line_indices: Vec::new(),
        }
    }

    pub fn flush_commands(&mut self) {
        let inner = self.inner.lock().unwrap();
        for _command in &inner.command_buffer {
            warn_once!("Debug drawing is currently broken :(");
        }
    }

    // TODO: This function is a hack to get debug pausing working. This should be better handled
    // by DebugDraw itself, rather than forcing Engine to handle it.
    pub fn clear_buffer(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.command_buffer.clear();
    }
}

impl Drop for DebugDraw {
    fn drop(&mut self) {
        unsafe {
            instance = ptr::null_mut();
        }
    }
}

/// Creates a mesh from a list of vertices and indices.
fn build_mesh(renderer: &mut Renderer, vertices: &[f32], indices: &[MeshIndex]) -> GpuMesh {
    let mesh = MeshBuilder::new()
    .set_position_data(Point::slice_from_f32_slice(vertices))
    .set_indices(indices)
    .build()
    .unwrap(); // TODO: Don't panic? I think in this case panicking is a bug in the engine and is pretty legit.

    renderer.register_mesh(&mesh)
}

#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    Line {
        start: Point,
        end: Point,
        color: Color,
    },
    Box {
        transform: Matrix4,
        color:     Color,
    },
    Sphere {
        center: Point,
        radius: f32,
        color: Color,
    }
}

#[derive(Debug)]
struct DebugDrawInner {
    command_buffer: Vec<DebugDrawCommand>,
}

pub fn draw_command(command: DebugDrawCommand) {
    debug_assert!(unsafe { !instance.is_null() }, "Cannot use debug drawing if there is no instance");

    let inner = unsafe { &*instance };
    let mut inner = inner.lock().unwrap();
    inner.command_buffer.push(command);
}

pub fn line(start: Point, end: Point) {
    draw_command(DebugDrawCommand::Line {
        start: start,
        end: end,
        color: color::WHITE,
    });
}

pub fn box_min_max(min: Point, max: Point) {
    box_min_max_color(min, max, color::WHITE);
}

pub fn box_min_max_color(min: Point, max: Point, color: Color) {
    let offset = max - min;
    let center = min + offset * 0.5;
    let transform = Matrix4::from_point(center) * Matrix4::from_scale_vector(offset);
    draw_command(DebugDrawCommand::Box {
        transform: transform,
        color: color,
    });
}

pub fn box_center_widths(center: Point, widths: Vector3) {
    let transform = Matrix4::from_point(center) * Matrix4::from_scale_vector(widths);
    draw_command(DebugDrawCommand::Box {
        transform: transform,
        color: color::WHITE,
    });
}

pub fn box_matrix(transform: Matrix4) {
    box_matrix_color(transform, color::WHITE);
}

pub fn box_matrix_color(transform: Matrix4, color: Color) {
    draw_command(DebugDrawCommand::Box {
        transform: transform,
        color: color,
    });
}

pub fn sphere(center: Point, radius: f32) {
    sphere_color(center, radius, color::WHITE);
}

pub fn sphere_color(center: Point, radius: f32, color: Color) {
    draw_command(DebugDrawCommand::Sphere {
        center: center,
        radius: radius,
        color: color,
    });
}
