use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, DrawIndexedIndirect},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, Color, CommandEncoderDescriptor,
    FragmentState, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, TextureViewDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState,
};

use crate::context::Context;

pub struct Renderer {
    pub context: Context,
    render_pipeline: RenderPipeline,
}

#[derive(Debug)]
pub struct RenderState {
    pub(crate) vertexes: Vec<Vec4>,
    pub(crate) indexes: Vec<u32>,
    pub(crate) instances: Vec<Instance>,
    pub(crate) draws: Vec<DrawIndexedIndirect>,
    pub(crate) clear: Color,
    pub(crate) camera: Mat4,
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Instance {
    pub(crate) transform: Mat4,
    pub(crate) color: Vec4,
}

impl Renderer {
    pub fn new(context: Context) -> Renderer {
        let render_pipeline = Renderer::create_pipeline(&context);

        Renderer {
            context,
            render_pipeline,
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

        let wgsl = wgpu::include_wgsl!("./wgsl/shader.wgsl");
        let shader = device.create_shader_module(wgsl);

        // Vertex Position
        let vertex_attributes = [VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }];

        // Instance
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
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 4 * (4 * 4), //bytes offset to 4th row
                shader_location: 5,
            },
        ];

        let vertex_buffer_layouts = [
            VertexBufferLayout {
                array_stride: 4 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attributes,
            },
            VertexBufferLayout {
                // byte size of Mat4(transform) + Vec4(color)
                array_stride: 5 * (4 * 4),
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

    fn create_camera_bind_group(
        context: &Context,
        layout: &BindGroupLayout,
        camera: Mat4,
    ) -> BindGroup {
        let transform_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(camera.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let entries = [BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        }];

        context.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        })
    }

    pub fn render<'a>(&'a self, render_state: RenderState) {
        let context = &self.context;

        // Setup buffers and bind groups
        let vertex_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&render_state.vertexes[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&render_state.indexes[..]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&render_state.instances[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let count = render_state.draws.len() as u32;

        let draws_buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(
                &render_state
                    .draws
                    .iter()
                    .map(|x| x.as_bytes())
                    .collect::<Vec<&[u8]>>()
                    .concat()[..],
            ),
            usage: wgpu::BufferUsages::INDIRECT,
        });

        let camera_layout = self.render_pipeline.get_bind_group_layout(0);
        let camera =
            Renderer::create_camera_bind_group(&context, &camera_layout, render_state.camera);

        // Start rendering phase
        let surface_texture = context.surface.get_current_texture().unwrap();
        let texture = &surface_texture.texture;
        let view = texture.create_view(&TextureViewDescriptor::default());

        let device = &context.device;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &context.multisample_texture_view,
                    resolve_target: Some(&view),
                    ops: Operations {
                        load: LoadOp::Clear(render_state.clear),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &camera, &[]);

            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, instance_buffer.slice(..));
            rpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            rpass.multi_draw_indexed_indirect(&draws_buffer, 0, count);
        }

        context.queue.submit(Some(encoder.finish()));
        surface_texture.present();
        context.window.request_redraw();
    }
}
