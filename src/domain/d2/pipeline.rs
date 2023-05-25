use std::num::NonZeroU64;

use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, IndexFormat,
    RenderPass, RenderPipeline, ShaderStages,
};

use crate::context::Context;

use super::geometry::GeometryBuffer;

pub struct D2 {
    pub render_pipeline: RenderPipeline,
    pub geometry_buffers: Vec<GeometryBuffer>,
}

impl D2 {
    pub fn new(context: &Context) -> D2 {
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(128),
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let vertex_attributes = [wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }];

        let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
            array_stride: 4 * 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        }];

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(context.texture_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 4,
                        ..Default::default()
                    },
                    multiview: None,
                });

        D2 {
            render_pipeline,
            geometry_buffers: vec![],
        }
    }

    pub fn add(&mut self, geometry_buffer: GeometryBuffer) {
        self.geometry_buffers.push(geometry_buffer);
    }

    pub fn render<'a>(&'a self, mut rpass: RenderPass<'a>) -> RenderPass<'a> {
        rpass.set_pipeline(&self.render_pipeline);

        for g in &self.geometry_buffers {
            rpass.set_vertex_buffer(0, g.vertex_buffer.slice(..));
            rpass.set_index_buffer(g.index_buffer.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..(g.index_buffer.size() as u32 / 4), 0, 0..1);
        }

        rpass
    }
}
