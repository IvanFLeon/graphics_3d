use wgpu::{Device, Instance, Queue, RenderPass, Surface, TextureFormat, TextureViewDescriptor};
use winit::window::Window;

pub struct Context {
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) texture_format: TextureFormat,
}

impl Context {
    pub async fn new(window: &Window, instance: &Instance) -> Self {
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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Context {
            surface,
            device,
            queue,
            texture_format,
        }
    }
    pub fn draw(self: Self, draw: impl Fn(RenderPass)) -> impl Fn() {
        move || {
            let surface_texture = self
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swapchain texture");

            let view = surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            draw(render_pass);

            self.queue.submit(Some(encoder.finish()));
            surface_texture.present();
        }
    }
}
