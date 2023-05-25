use std::num::NonZeroU64;

use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, RenderPass,
    RenderPipeline, ShaderStages,
};

use crate::context::Context;

use super::Renderer;

pub struct D2(RenderPipeline);

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

        D2(render_pipeline)
    }

    pub fn renderer<'a>(
        &'a self,
        render_pass: impl Into<RenderPass<'a>>,
        context: &'a Context,
    ) -> Renderer {
        let D2(render_pipeline) = self;
        let mut render_pass = render_pass.into();
        render_pass.set_pipeline(render_pipeline);
        Renderer(render_pass)
    }
}
