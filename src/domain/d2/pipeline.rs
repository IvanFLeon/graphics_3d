use glam::{Mat4, Vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, FragmentState, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, VertexAttribute, VertexBufferLayout, VertexState,
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
        let render_pipeline = D2::create_pipeline(context);
        let transform_layout = &render_pipeline.get_bind_group_layout(0);
        let transform = D2::create_transform_bind_group(context, transform_layout);

        D2 {
            render_pipeline,
            transform,
            geometry_buffers: vec![],
            context: context.clone(),
        }
    }

    fn create_pipeline(context: &Context) -> RenderPipeline {
        let context = context.read().unwrap();
        let device = &context.device;

        let binding_type = BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(64),
        };

        let entries = &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: binding_type,
            count: None,
        }];

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let wgsl = wgpu::include_wgsl!("shader.wgsl");
        let shader = device.create_shader_module(wgsl);

        let vertex_attributes = [VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }];

        let vertex_buffer_layouts = [VertexBufferLayout {
            array_stride: 4 * 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        }];

        let vertex_state = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &vertex_buffer_layouts,
        };

        let fragment_state = FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(context.texture_format.into())],
        };

        let multisample_state = MultisampleState {
            count: 4,
            ..Default::default()
        };

        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            primitive: PrimitiveState::default(),
            multisample: multisample_state,
            depth_stencil: None,
            multiview: None,
        };

        device.create_render_pipeline(&descriptor)
    }

    fn create_transform_bind_group(context: &Context, layout: &BindGroupLayout) -> BindGroup {
        let context = context.read().unwrap();
        let device = &context.device;
        let surface_config = &context.surface_config;

        let half_width = surface_config.width as f32 / 2.;
        let half_height = surface_config.height as f32 / 2.;

        let orthographic =
            Mat4::orthographic_rh_gl(-half_width, half_width, -half_height, half_height, -2., 0.);

        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(orthographic.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let entries = [BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        }];

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        })
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
