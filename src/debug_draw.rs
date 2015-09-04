use std::rc::Rc;

use math::*;
use polygon::Camera;
use polygon::gl_render::{GLRender, ShaderProgram, GLMeshData};
use polygon::geometry::Mesh;
use resource::ResourceManager;

#[derive(Debug)]
pub struct DebugDraw {
    renderer: Rc<GLRender>,
    command_buffer: Vec<DebugDrawCommand>,

    shader: ShaderProgram,
    unit_cube: GLMeshData,
}

impl Clone for DebugDraw {
    fn clone(&self) -> DebugDraw {
        DebugDraw {
            renderer: self.renderer.clone(),
            command_buffer: self.command_buffer.clone(),

            shader: self.shader.clone(),
            unit_cube: build_mesh(&*self.renderer, &CUBE_VERTS, &CUBE_INDICES)
        }
    }
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
    pub fn new(renderer: Rc<GLRender>, resource_manager: Rc<ResourceManager>) -> DebugDraw {
        DebugDraw {
            renderer: renderer.clone(),
            shader: resource_manager.get_shader("shaders/debug_draw.glsl").unwrap(),
            command_buffer: Vec::new(),
            unit_cube: build_mesh(&*renderer, &CUBE_VERTS, &CUBE_INDICES),
        }
    }

    pub fn draw_command(&mut self, command: DebugDrawCommand) {
        self.command_buffer.push(command);
    }

    pub fn draw_line(&mut self, start: Point, end: Point) {
        self.draw_command(DebugDrawCommand::Line {
            start: start,
            end: end,
        });
    }

    pub fn draw_box_min_max(&mut self, min: Point, max: Point) {
        let diff = max - min;

        self.draw_command(DebugDrawCommand::Box {
            center: min + diff  * 0.5,
            widths: diff,
        });
    }

    pub fn draw_box_center_widths(&mut self, center: Point, widths: Vector3) {
        self.draw_command(DebugDrawCommand::Box {
            center: center,
            widths: widths,
        });
    }

    pub fn flush_commands(&mut self, camera: &Camera) {
        for command in &self.command_buffer {
            match command {
                &DebugDrawCommand::Line { start, end } => {
                    self.renderer.draw_line(camera, &self.shader, start, end);
                },
                &DebugDrawCommand::Box { center, widths } => {
                    let model_transform =
                        Matrix4::from_point(center) * Matrix4::from_scale_vector(widths);
                    self.renderer.draw_wireframe(camera, &self.shader, &self.unit_cube, model_transform);
                }
            }
        }

        self.command_buffer.clear();
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
    }
}
