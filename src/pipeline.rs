use glam::{Mat4, Vec3, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, DrawIndexedIndirect},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, FragmentState, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, VertexAttribute, VertexBufferLayout, VertexState,
};

use crate::context::Context;

use super::{
    circle,
    geometry::{square, triangle, triangle_strip, TriangleStrip},
    polygon,
};

pub struct Renderer<'a> {
    pub render_pipeline: RenderPipeline,
    // Local Geometry Data
    pub vertexes: Vec<Vec4>,
    pub indexes: Vec<u32>,
    pub instances: Vec<Mat4>,
    pub draws: Vec<DrawIndexedIndirect>,
    // Geometry Buffers
    pub vertex_buffer: Option<Buffer>,
    pub instance_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub draws_buffer: Option<Buffer>,
    // Stack
    pub stack: Vec<TransformInstance>,
    // World/Camera
    pub transform: BindGroup,
}

#[derive(Clone, Copy)]
pub struct TransformInstance {
    transform: Mat4,
    index: Option<usize>,
}

impl<'a> D2<'a> {
    pub fn new(context: Context) -> D2 {
        let render_pipeline = D2::create_pipeline(context);
        let transform_layout = &render_pipeline.get_bind_group_layout(0);
        let transform = D2::create_transform_bind_group(context, transform_layout);
        let stack = vec![TransformInstance {
            transform: Mat4::IDENTITY,
            index: None,
        }];

        D2 {
            render_pipeline,
            vertexes: vec![],
            indexes: vec![],
            instances: vec![],
            draws: vec![],
            vertex_buffer: None,
            instance_buffer: None,
            index_buffer: None,
            draws_buffer: None,
            stack,
            transform,
        }
    }

    fn create_pipeline(context: &Context) -> RenderPipeline {
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

        //Vertex Position
        let vertex_attributes = [VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }];

        //Local Transformation Matrix (Instanced)
        let transform_attributes = [
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0, //bytes no offset to 1st row
                shader_location: 1,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: (4 * 4), //bytes offset to 2nd row
                shader_location: 2,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 2 * (4 * 4), //bytes offset to 3rd row
                shader_location: 3,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 3 * (4 * 4), //bytes offset to 4th row
                shader_location: 4,
            },
        ];

        let vertex_buffer_layouts = [
            VertexBufferLayout {
                array_stride: 4 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attributes,
            },
            VertexBufferLayout {
                array_stride: 4 * (4 * 4), //byte size of Mat4
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &transform_attributes,
            },
        ];

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
        self.vertexes.clear();
        self.indexes.clear();
        self.instances.clear();
        self.draws.clear();
        self.stack.clear();
        self.stack.push(TransformInstance {
            transform: Mat4::IDENTITY,
            index: None,
        });
    }
    pub fn triangle(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32)) {
        self.add(triangle(a, b, c));
    }

    pub fn square(&mut self) {
        self.add(square());
    }

    pub fn circle(&mut self) {
        self.add(circle());
    }

    pub fn polygon(&mut self, res: u32) {
        self.add(polygon(res));
    }

    pub fn triangle_strip(&mut self, call: impl FnOnce(&mut TriangleStrip)) {
        self.add(triangle_strip(call));
    }

    pub fn add(&mut self, geometry: (Vec<Vec4>, Vec<u32>)) {
        let (mut vertexes, mut indexes) = geometry;

        let last = self.stack.last_mut().unwrap();

        let index = if let Some(index) = last.index {
            index
        } else {
            self.instances.push(last.transform);
            last.index = Some(self.instances.len() - 1);
            last.index.unwrap()
        };

        let draw = DrawIndexedIndirect {
            vertex_offset: self.vertexes.len() as i32,
            base_index: self.indexes.len() as u32,
            base_instance: (index) as u32,
            vertex_count: indexes.len() as u32,
            instance_count: 1,
        };

        self.vertexes.append(&mut vertexes);
        self.indexes.append(&mut indexes);
        self.draws.push(draw);
    }

    pub fn push(&mut self) {
        let instance = *self.stack.last().clone().unwrap();
        self.stack.push(instance);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        let instance = self.stack.last_mut().unwrap();

        let vec = Vec3::new(x, y, 0.);
        instance.transform = instance.transform.mul_mat4(&Mat4::from_translation(vec));
        instance.index = None;
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        let instance = self.stack.last_mut().unwrap();

        let vec = Vec3::new(x, y, 0.);
        instance.transform = instance.transform.mul_mat4(&Mat4::from_scale(vec));
        instance.index = None;
    }

    pub fn rotate(&mut self, th: f32) {
        let instance = self.stack.last_mut().unwrap();

        instance.transform = instance.transform.mul_mat4(&Mat4::from_rotation_z(th));
        instance.index = None;
    }

    pub fn identity(&mut self) {
        let instance = self.stack.last_mut().unwrap();

        instance.transform = Mat4::IDENTITY;
        instance.index = None;
    }

    pub fn render(&'a mut self, mut rpass: RenderPass<'a>) -> RenderPass {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.transform, &[]);

        //Create buffers
        self.vertex_buffer = Some(
            self.context
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.vertexes[..]),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        );

        self.instance_buffer = Some(self.context.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.instances[..]),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        self.index_buffer = Some(
            self.context
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.indexes[..]),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );

        let contents = &self
            .draws
            .iter()
            .map(|f| f.as_bytes())
            .collect::<Vec<_>>()
            .concat();

        self.draws_buffer = Some(
            self.context
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&contents[..]),
                    usage: wgpu::BufferUsages::INDIRECT,
                }),
        );

        let vertex_buffer_slice = self.vertex_buffer.as_ref().unwrap().slice(..);
        let instance_buffer_slice = self.instance_buffer.as_ref().unwrap().slice(..);
        let index_buffer_slice = self.index_buffer.as_ref().unwrap().slice(..);
        let draw_buffer = self.draws_buffer.as_ref().unwrap();

        let count = self.draws.len() as u32;

        rpass.set_vertex_buffer(0, vertex_buffer_slice);
        rpass.set_vertex_buffer(1, instance_buffer_slice);
        rpass.set_index_buffer(index_buffer_slice, IndexFormat::Uint32);
        rpass.multi_draw_indexed_indirect(draw_buffer, 0, count);

        rpass
    }
}
