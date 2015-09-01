use std::rc::Rc;

use math::*;
use polygon::gl_render::GLRender;

#[derive(Debug, Clone)]
pub struct DebugDraw {
    renderer: Rc<GLRender>,
    command_buffer: Vec<DebugDrawCommand>,
}

impl DebugDraw {
    pub fn new(renderer: Rc<GLRender>) -> DebugDraw {
        DebugDraw {
            renderer: renderer,
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
}

#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    Line {
        start: Point,
        end: Point,
    }
}
