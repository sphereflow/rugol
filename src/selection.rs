use crate::render_mini::{Vec2, Vertex};
use miniquad::{window::screen_size, Bindings, BufferSource, BufferType, BufferUsage, Context};

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Vec2,
    pub end: Option<Vec2>,
    pub line_width: f32,
    binding: Bindings,
}

impl Selection {
    pub fn new(ctx: &mut Context, start: Vec2) -> Selection {
        let vertices: Vec<Vertex> = Vec::new();
        let vb = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            miniquad::BufferSource::Empty {
                size: vertices.len(),
                element_size: std::mem::size_of::<Vertex>(),
            },
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vec![
                0, 1, 5, 0, 5, 4, 5, 1, 6, 6, 1, 2, 6, 2, 7, 7, 2, 3, 7, 3, 4, 3, 0, 4,
            ]),
        );
        Selection {
            binding: Bindings {
                vertex_buffers: vec![vb],
                index_buffer,
                images: vec![],
            },
            start,
            end: None,
            line_width: 3.0,
        }
    }

    // regenerate vertices because something might have changed
    pub fn update_binding(&mut self, ctx: &mut Context) {
        if let Some(end) = self.end {
            let (screen_width, screen_height) = screen_size();
            let l = 2.0 * (self.start.x.min(end.x) / screen_width) - 1.0;
            let r = 2.0 * (self.start.x.max(end.x) / screen_width) - 1.0;
            let b = 2.0 * (self.start.y.min(end.y) / screen_height) - 1.0;
            let t = 2.0 * (self.start.y.max(end.y) / screen_height) - 1.0;
            let ww = self.line_width / screen_width;
            let wh = self.line_width / screen_height;
            let color: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
            let vertices: Vec<Vertex> = vec![
                Vertex::new(l, t, color),
                Vertex::new(r, t, color),
                Vertex::new(r, b, color),
                Vertex::new(l, b, color),
                Vertex::new(l + ww, t - wh, color),
                Vertex::new(r - ww, t - wh, color),
                Vertex::new(r - ww, b + wh, color),
                Vertex::new(l + ww, b + wh, color),
            ];
            let buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );
            self.binding.vertex_buffers[0] = buffer;
        } else {
            let buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice::<Vertex>(&[]),
            );
            self.binding.vertex_buffers[0] = buffer;
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        ctx.apply_bindings(&self.binding);
        // start index: 0
        // 4 lines * 2 triangles per line * 3 indices per triangle
        // 1 instance
        ctx.draw(0, 4 * 2 * 3, 1);
    }
}
