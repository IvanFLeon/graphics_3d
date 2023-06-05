use std::sync::{Arc, RwLock};

use wgpu::{
    CommandEncoderDescriptor, Device, Extent3d, Instance, LoadOp, Operations, PowerPreference,
    Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
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

pub type Context = Arc<RwLock<SharedContext>>;

impl SharedContext {
    pub async fn new(window: Window) -> Self {
        let instance = Instance::default();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let options = RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };

        let adapter = instance.request_adapter(&options).await.unwrap();

        let descriptor = wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::MULTI_DRAW_INDIRECT,
            limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        };

        let (device, queue) = adapter.request_device(&descriptor, None).await.unwrap();

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

        SharedContext {
            window,
            surface,
            surface_config,
            device,
            queue,
            texture_format,
            multisample_texture_view,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.multisample_texture_view =
            create_multisample_texture_view(&self.device, &self.surface_config);
    }
}

pub fn draw(context: &Context, mut fn_draw: impl FnMut(RenderPass, u32)) -> impl FnMut(u32) {
    let context = context.clone();

    move |frame| {
        let context = context.read().unwrap();

        let queue = &context.queue;
        let device = &context.device;
        let window = &context.window;
        let surface = &context.surface;
        let multisample_texture_view = &context.multisample_texture_view;

        let descriptor = TextureViewDescriptor::default();
        let surface_texture = surface.get_current_texture().unwrap();
        let texture = &surface_texture.texture;
        let view = texture.create_view(&descriptor);

        let descriptor = CommandEncoderDescriptor::default();
        let mut encoder = device.create_command_encoder(&descriptor);

        let operations = Operations {
            load: LoadOp::Clear(wgpu::Color::WHITE),
            store: true,
        };

        let color_attachment = RenderPassColorAttachment {
            view: &multisample_texture_view,
            resolve_target: Some(&view),
            ops: operations,
        };

        let descriptor = RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
        };

        let render_pass = encoder.begin_render_pass(&descriptor);

        fn_draw(render_pass, frame);

        queue.submit(Some(encoder.finish()));
        surface_texture.present();
        window.request_redraw();
    }
}

fn create_multisample_texture_view(device: &Device, config: &SurfaceConfiguration) -> TextureView {
    let size = Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let texture_descriptor = TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format: config.view_formats[0],
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let view_descriptor = TextureViewDescriptor::default();

    device
        .create_texture(&texture_descriptor)
        .create_view(&view_descriptor)
}
