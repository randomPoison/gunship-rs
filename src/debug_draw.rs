use std::rc::Rc;

use math::*;
use polygon::Camera;
use polygon::gl_render::{GLRender, ShaderProgram};
use resource::ResourceManager;

#[derive(Debug, Clone)]
pub struct DebugDraw {
    renderer: Rc<GLRender>,
    shader: ShaderProgram,
    command_buffer: Vec<DebugDrawCommand>,
}

impl DebugDraw {
    pub fn new(renderer: Rc<GLRender>, resource_manager: Rc<ResourceManager>) -> DebugDraw {
        DebugDraw {
            renderer: renderer,
            shader: resource_manager.get_shader("shaders/debug_draw.glsl").unwrap(),
            command_buffer: Vec::new(),
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

    pub fn flush_commands(&mut self, camera: &Camera) {
        for command in &self.command_buffer {
            match command {
                &DebugDrawCommand::Line { start, end } => {
                    self.renderer.draw_line(camera, &self.shader, start, end);
                }
            }
        }

        self.command_buffer.clear();
    }
}

#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    Line {
        start: Point,
        end: Point,
    }
}
