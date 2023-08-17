use crate::context::Context;

use app::App;
use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub mod app;
pub mod color;
pub mod context;
pub mod renderer;
pub mod state;

pub async fn run(redraw: fn(&mut App)) {
    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop);
    let window = window.expect("WindowCreateFailed");

    let context = Context::new(window).await;
    let mut renderer = Renderer::new(context);

    event_loop.run(move |event, _target, control_flow| {
        control_flow.set_poll();
        let mut app = App::new(
            renderer.context.surface_config.width,
            renderer.context.surface_config.height,
        );

        match event {
            Event::RedrawRequested(_) => {
                redraw(&mut app);
                renderer.render(app.state.serialize());
                app.frame += 1;
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(size),
            } => {
                renderer.context.resize(size);
                app.size.width = size.width;
                app.size.height = size.height;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            _ => (),
        };
    });
}
