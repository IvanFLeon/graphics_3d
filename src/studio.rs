use std::collections::HashMap;

use wgpu::Instance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use crate::context::Context;

pub struct Studio {
    event_loop: EventLoop<()>,
    windows: HashMap<WindowId, Window>,
    draws: HashMap<WindowId, Box<dyn Fn()>>,
}

impl Studio {
    pub fn run(self: Self) {
        self.event_loop.run(move |event, _target, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                }
                Event::RedrawRequested(window_id) => {
                    let draw = self
                        .draws
                        .get(&window_id)
                        .expect("Couldn't find draw callback.");
                    draw();
                }

                _ => (),
            };
        });
    }

    pub async fn canvas<F: Fn() + 'static>(self: &mut Self, setup: fn(Context) -> F) {
        let window = Window::new(&self.event_loop).expect("Couldn't create window.");
        let instance = Instance::default();
        let context = Context::new(&window, &instance).await;

        let draw = setup(context);

        self.draws.insert(window.id(), Box::new(draw));
        self.windows.insert(window.id(), window);
    }
}

impl Default for Studio {
    fn default() -> Self {
        Studio {
            event_loop: EventLoop::new(),
            windows: HashMap::new(),
            draws: HashMap::new(),
        }
    }
}
