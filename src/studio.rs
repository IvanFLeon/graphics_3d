use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use crate::context::{Context, SharedContext};

pub struct Studio {
    event_loop: EventLoop<()>,
    canvases: HashMap<WindowId, Canvas>,
}

struct Canvas {
    context: Context,
    redraw: Box<dyn FnMut(u32)>,
    frame: u32,
}

impl Studio {
    pub fn run(mut self: Self) {
        self.event_loop.run(move |event, _target, control_flow| {
            control_flow.set_poll();

            match event {
                Event::RedrawRequested(window_id) => {
                    let canvas = self.canvases.get_mut(&window_id);
                    let mut canvas = canvas.expect("CanvasNotFoundForWindowId");

                    (canvas.redraw)(canvas.frame);
                    canvas.frame += 1;
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Resized(size),
                } => {
                    let canvas = self.canvases.get_mut(&window_id);
                    let canvas = canvas.expect("CanvasNotFoundForWindowId");

                    let context = canvas.context.write();
                    let mut context = context.expect("ContextWriteLockFailed");

                    context.resize(size);
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

    pub async fn canvas<F: FnMut(u32) + 'static>(self: &mut Self, setup: fn(Context) -> F) {
        let window = Window::new(&self.event_loop);
        let window = window.expect("WindowCreateFailed");
        let window_id = window.id();

        let context = Arc::new(RwLock::new(SharedContext::new(window).await));
        let redraw = Box::new(setup(context.clone()));

        self.canvases.insert(
            window_id,
            Canvas {
                context,
                redraw,
                frame: 0,
            },
        );
    }
}

impl Default for Studio {
    fn default() -> Self {
        Studio {
            event_loop: EventLoop::new(),
            canvases: HashMap::new(),
        }
    }
}
