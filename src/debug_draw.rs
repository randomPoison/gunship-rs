use std::rc::Rc;
use std::ptr;
use std::f32::consts::PI;

use math::*;
use polygon::Camera;
use polygon::gl_render::{GLRender, ShaderProgram, GLMeshData};
use polygon::geometry::Mesh;
use resource::ResourceManager;

static mut instance: *mut DebugDrawInner = 0 as *mut DebugDrawInner;

#[derive(Debug)]
pub struct DebugDraw {
    renderer: Rc<GLRender>,

    shader: ShaderProgram,
    unit_cube: GLMeshData,
    unit_sphere: GLMeshData,

    inner: Box<DebugDrawInner>,

    // Vecs used for dynamically reconstructing meshes.
    line_vertices: Vec<f32>,
    line_indices: Vec<u32>,
}

static CUBE_VERTS: [f32; 32] =
    [ 0.5,  0.5,  0.5, 1.0,
      0.5,  0.5, -0.5, 1.0,
      0.5, -0.5,  0.5, 1.0,
      0.5, -0.5, -0.5, 1.0,
     -0.5,  0.5,  0.5, 1.0,
     -0.5,  0.5, -0.5, 1.0,
     -0.5, -0.5,  0.5, 1.0,
     -0.5, -0.5, -0.5, 1.0,];
static CUBE_INDICES: [u32; 24] =
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

impl DebugDraw {
    pub fn new(renderer: Rc<GLRender>, resource_manager: &ResourceManager) -> DebugDraw {
        assert!(unsafe { instance.is_null() }, "Cannot create more than one instance of DebugDraw at a time");

        let mut inner = Box::new(DebugDrawInner {
            command_buffer: Vec::new(),
        });

        unsafe {
            instance = &mut *inner;
        }

        // Build sphere vertices. This could be done offline, but I'm lazy so we do it at runtime!
        let unit_sphere = {
            const VERTS_PER_CIRCLE: usize = 50;
            let mut sphere_verts = Vec::new();
            let mut sphere_indices = Vec::<u32>::new();

            // Vertices around X axis.
            sphere_verts.push(Point::new(0.0, 0.0, 1.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(0.0, theta.sin(), theta.cos()));
                sphere_indices.push(sphere_verts.len() as u32 - 2);
                sphere_indices.push(sphere_verts.len() as u32 - 1);
            }

            // Vertices around Y axis.
            sphere_verts.push(Point::new(1.0, 0.0, 0.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(theta.cos(), 0.0, theta.sin()));
                sphere_indices.push(sphere_verts.len() as u32 - 2);
                sphere_indices.push(sphere_verts.len() as u32 - 1);
            }

            // Vertices around Z axis.
            sphere_verts.push(Point::new(1.0, 0.0, 0.0));
            for offset in 0..VERTS_PER_CIRCLE {
                let percent = offset as f32 / VERTS_PER_CIRCLE as f32;
                let theta = percent * 2.0 * PI;
                sphere_verts.push(Point::new(theta.cos(), theta.sin(), 0.0));
                sphere_indices.push(sphere_verts.len() as u32 - 2);
                sphere_indices.push(sphere_verts.len() as u32 - 1);
            }

            build_mesh(&*renderer, Point::as_ref(&*sphere_verts), &*sphere_indices)
        };

        DebugDraw {
            renderer: renderer.clone(),

            shader: resource_manager.get_shader("shaders/debug_draw.glsl").unwrap(),
            unit_cube: build_mesh(&*renderer, &CUBE_VERTS, &CUBE_INDICES),
            unit_sphere: unit_sphere,

            inner: inner,

            line_vertices: Vec::new(),
            line_indices: Vec::new(),
        }
    }

    pub fn flush_commands(&mut self, camera: &Camera) {
        for command in &self.inner.command_buffer {
            match command {
                &DebugDrawCommand::Line { start, end } => {
                    self.line_vertices.extend(start.as_array());
                    self.line_vertices.extend(end.as_array());
                },
                &DebugDrawCommand::Box { center, widths } => {
                    let model_transform =
                        Matrix4::from_point(center) * Matrix4::from_scale_vector(widths);
                    self.renderer.draw_wireframe(camera, &self.shader, &self.unit_cube, model_transform);
                },
                &DebugDrawCommand::Sphere { center, radius } => {
                    let model_transform =
                        Matrix4::from_point(center) * Matrix4::scale(radius, radius, radius);
                    self.renderer.draw_wireframe(camera, &self.shader, &self.unit_sphere, model_transform);
                },
            }
        }

        if !self.line_vertices.is_empty() {
            for index in 0..self.line_vertices.len() / 4 {
                self.line_indices.push(index as u32);
            }
            let line_mesh = build_mesh(&*self.renderer, &self.line_vertices, &self.line_indices);
            self.renderer.draw_wireframe(camera, &self.shader, &line_mesh, Matrix4::identity());
            self.renderer.delete_mesh(line_mesh);
        }

        self.line_vertices.clear();
        self.line_indices.clear();
    }

    // TODO: This function is a hack to get debug pausing working. This should be better handled
    // by DebugDraw itself, rather than forcing Engine to handle it.
    pub fn clear_buffer(&mut self) {
        self.inner.command_buffer.clear();
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
fn build_mesh(renderer: &GLRender, vertices: &[f32], indices: &[u32]) -> GLMeshData {
    let mesh = Mesh::from_raw_data(vertices, indices);
    renderer.gen_mesh(&mesh)
}

#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    Line {
        start: Point,
        end: Point,
    },
    Box {
        center: Point,
        widths: Vector3,
    },
    Sphere {
        center: Point,
        radius: f32,
    }
}

#[derive(Debug)]
struct DebugDrawInner {
    command_buffer: Vec<DebugDrawCommand>,
}

pub fn draw_command(command: DebugDrawCommand) {
    debug_assert!(unsafe { !instance.is_null() }, "Cannot use debug drawing if there is no instance");

    let inner = unsafe { &mut *instance };
    inner.command_buffer.push(command);
}

pub fn line(start: Point, end: Point) {
    draw_command(DebugDrawCommand::Line {
        start: start,
        end: end,
    });
}

pub fn box_min_max(min: Point, max: Point) {
    let diff = max - min;
    draw_command(DebugDrawCommand::Box {
        center: min + diff  * 0.5,
        widths: diff,
    });
}

pub fn box_center_widths(center: Point, widths: Vector3) {
    draw_command(DebugDrawCommand::Box {
        center: center,
        widths: widths,
    });
}

pub fn sphere(center: Point, radius: f32) {
    draw_command(DebugDrawCommand::Sphere {
        center: center,
        radius: radius,
    });
}
