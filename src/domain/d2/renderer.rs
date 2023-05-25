use wgpu::{IndexFormat, RenderPass};

use super::geometry::GeometryBuffer;

pub struct Renderer<'a>(pub RenderPass<'a>);

impl<'a> Renderer<'a> {
    pub fn transform(&mut self) {}

    pub fn draw(&mut self, geometry_buffer: &'a GeometryBuffer) {
        let Renderer(rpass) = self;

        rpass.set_vertex_buffer(0, geometry_buffer.vertex_buffer.slice(..));
        rpass.set_index_buffer(geometry_buffer.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..(geometry_buffer.index_buffer.size() as u32 / 4), 0, 0..1);
    }
}

impl<'a> From<Renderer<'a>> for RenderPass<'a> {
    fn from(value: Renderer<'a>) -> Self {
        let Renderer(rpass) = value;
        rpass
    }
}
