use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use crate::context::Context;

pub struct Studio {
    event_loop: EventLoop<()>,
    contexts: HashMap<WindowId, Context>,
    draws: HashMap<WindowId, Box<dyn FnMut(u32)>>,
}

impl Studio {
    pub fn run(mut self: Self) {
        let mut frame: u32 = 0;

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
                    let ctx = self.contexts.get_mut(&window_id).unwrap();
                    ctx.resize(size);
                }
                Event::RedrawRequested(window_id) => {
                    let draw = self.draws.get_mut(&window_id).unwrap();
                    draw(frame);

                    frame += 1;
                }

                _ => (),
            };
        });
    }

    pub async fn canvas<F: FnMut(u32) + 'static>(self: &mut Self, setup: fn(Context) -> F) {
        let window = Window::new(&self.event_loop).expect("Couldn't create window.");
        let window_id = window.id();

        let context = Context::new(window).await;
        self.contexts.insert(window_id, context.clone());

        let draw = setup(context.clone());
        self.draws.insert(window_id, Box::new(draw));
    }
}

impl Default for Studio {
    fn default() -> Self {
        Studio {
            event_loop: EventLoop::new(),
            contexts: HashMap::new(),
            draws: HashMap::new(),
        }
    }
}
