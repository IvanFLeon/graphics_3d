use glam::{Mat4, Vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, IndexFormat, RenderPass, RenderPipeline,
    ShaderStages,
};

use crate::context::Context;

use super::{circle, geometry::GeometryBuffer};

pub struct D2 {
    pub render_pipeline: RenderPipeline,
    pub geometry_buffers: Vec<GeometryBuffer>,
    pub transform: BindGroup,
    pub context: Context,
}

impl D2 {
    pub fn new(context: &Context) -> D2 {
        let shared = context.read().unwrap();

        let bind_group_layout =
            shared
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(64),
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            shared
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let shader = shared
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
            shared
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
                        targets: &[Some(shared.texture_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 4,
                        ..Default::default()
                    },
                    multiview: None,
                });

        let half_width = shared.surface_config.width as f32 / 2.;
        let half_height = shared.surface_config.height as f32 / 2.;

        let orthographic =
            Mat4::orthographic_rh_gl(-half_width, half_width, -half_height, half_height, -2., 0.);

        let transform_buffer = shared.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(orthographic.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let transform = shared.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        drop(shared);

        D2 {
            render_pipeline,
            transform,
            geometry_buffers: vec![],
            context: context.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.geometry_buffers.clear();
    }

    pub fn push(&mut self, geometry_buffer: GeometryBuffer) {
        self.geometry_buffers.push(geometry_buffer);
    }

    pub fn circle(&mut self, res: u32) {
        self.push(circle(res).buffer(&self.context));
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let context = self.context.read().unwrap();

        let half_width = context.surface_config.width as f32 / 2.;
        let half_height = context.surface_config.height as f32 / 2.;
        let mat =
            Mat4::orthographic_rh_gl(-half_width, half_width, -half_height, half_height, -2., 0.);
        let mat = mat * Mat4::from_translation(Vec3::new(x, y, z));

        let transform_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mat.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let transform = context.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.render_pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        self.transform = transform;
    }

    pub fn render<'a>(&'a self, mut rpass: RenderPass<'a>) -> RenderPass {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.transform, &[]);

        for g in &self.geometry_buffers {
            rpass.set_vertex_buffer(0, g.vertex_buffer.slice(..));
            rpass.set_index_buffer(g.index_buffer.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..(g.index_buffer.size() as u32 / 4), 0, 0..1);
        }

        rpass
    }
}
