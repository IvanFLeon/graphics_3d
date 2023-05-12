use std::f32::consts::PI;

use glam::Vec4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, IndexFormat, RenderPass,
};

use crate::context::Context;

#[derive(Debug)]
pub struct Circle {
    vertexes: Vec<Vec4>,
    indexes: Vec<u32>,
}

pub struct CircleBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl Circle {
    pub fn new(res: u32) -> Self {
        let mut vertexes: Vec<Vec4> = vec![];
        let mut indexes: Vec<u32> = vec![];

        vertexes.push(Vec4::new(0., 0., 0., 0.));

        let ph = (PI * 2.) / (res as f32);
        let mut th = 0.;
        for i in 1..=res {
            vertexes.push(Vec4::new(f32::cos(th) * 0.5, f32::sin(th) * 0.5, 0., 1.));

            indexes.push(0);
            indexes.push(i);
            indexes.push(i + 1);

            th += ph;
        }

        indexes.pop();
        indexes.push(1);

        Self { vertexes, indexes }
    }

    pub fn buffer(&self, context: &Context) -> CircleBuffer {
        CircleBuffer {
            vertex_buffer: context.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertexes[..]),
                usage: wgpu::BufferUsages::VERTEX,
            }),

            index_buffer: context.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.indexes[..]),
                usage: wgpu::BufferUsages::INDEX,
            }),
        }
    }
}

impl CircleBuffer {
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    }
}
