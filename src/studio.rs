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

impl Studio {
    pub fn run(mut self: Self) {
        self.event_loop.run(move |event, _target, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Resized(size),
                } => {
                    let canvas = self.canvases.get_mut(&window_id).unwrap();
                    let mut shared = canvas.context.write().unwrap();
                    shared.resize(size);
                }
                Event::RedrawRequested(window_id) => {
                    let mut canvas = self.canvases.get_mut(&window_id).unwrap();
                    (canvas.redraw)(canvas.frame);
                    canvas.frame += 1;
                }
                _ => (),
            };
        });
    }

    pub async fn canvas<F: FnMut(u32) + 'static>(self: &mut Self, setup: fn(Context) -> F) {
        let window = Window::new(&self.event_loop).expect("Couldn't create window.");
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

struct Canvas {
    context: Context,
    redraw: Box<dyn FnMut(u32)>,
    frame: u32,
}

impl Default for Studio {
    fn default() -> Self {
        Studio {
            event_loop: EventLoop::new(),
            canvases: HashMap::new(),
        }
    }
}
