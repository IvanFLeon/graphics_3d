use std::sync::{Arc, RwLock};

use wgpu::{
    Device, Instance, Queue, RenderPass, Surface, SurfaceConfiguration, TextureDescriptor,
    TextureFormat, TextureView, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct SharedContext {
    pub(crate) window: Window,
    pub(crate) surface: Surface,
    pub(crate) surface_config: SurfaceConfiguration,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) texture_format: TextureFormat,
    pub(crate) multisample_texture_view: TextureView,
}

#[derive(Clone)]
pub struct Context(pub Arc<RwLock<SharedContext>>);

impl Context {
    pub async fn new(window: Window) -> Self {
        let instance = Instance::default();

        let surface = unsafe { instance.create_surface(&window) }
            .expect("Surface not supported by any backend.");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No adapter found with the specified requirements.");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::MULTI_DRAW_INDIRECT,
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        //Swapchain
        let surface_capabilities = surface.get_capabilities(&adapter);
        let texture_format = surface_capabilities.formats[0];

        let size = window.inner_size();

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![texture_format],
        };

        surface.configure(&device, &surface_config);

        let multisample_texture_view = create_multisample_texture_view(&device, &surface_config);

        Context(Arc::new(RwLock::new(SharedContext {
            window,
            surface,
            surface_config,
            device,
            queue,
            texture_format,
            multisample_texture_view,
        })))
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let mut ctx = self.0.write().unwrap();
        ctx.surface_config.width = size.width;
        ctx.surface_config.height = size.height;
        ctx.surface.configure(&ctx.device, &ctx.surface_config);
        ctx.multisample_texture_view =
            create_multisample_texture_view(&ctx.device, &ctx.surface_config);
    }

    pub fn draw(&self, mut draw: impl FnMut(RenderPass, u32)) -> impl FnMut(u32) {
        let ctx = self.0.clone();

        move |frame| {
            let ctx = ctx.read().unwrap();

            let surface_texture = ctx
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swapchain texture");

            let view = surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ctx.multisample_texture_view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            draw(render_pass, frame);

            ctx.queue.submit(Some(encoder.finish()));
            surface_texture.present();

            ctx.window.request_redraw();
        }
    }
}

fn create_multisample_texture_view(device: &Device, config: &SurfaceConfiguration) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: config.view_formats[0],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}
