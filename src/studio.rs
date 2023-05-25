use std::collections::HashMap;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use crate::context::Context;

pub struct Studio {
    event_loop: EventLoop<()>,
    contexts: HashMap<WindowId, Context>,
    redraws: HashMap<WindowId, Box<dyn FnMut(&Context, u32)>>,
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
                    let ctx = self
                        .contexts
                        .get_mut(&window_id)
                        .expect("Couldn't retrieve Context for window_id.");

                    ctx.resize(size);
                }
                Event::RedrawRequested(window_id) => {
                    let context = self
                        .contexts
                        .get(&window_id)
                        .expect("Couldn't retrieve Context for window_id.");

                    let redraw = self
                        .redraws
                        .get_mut(&window_id)
                        .expect("Couldn't retrieve Redraw call for window_id.");

                    redraw(&context, frame);

                    frame += 1;
                    context.window.request_redraw();
                }

                _ => (),
            };
        });
    }

    pub async fn canvas<F: FnMut(&Context, u32) + 'static>(
        self: &mut Self,
        setup: fn(&Context) -> F,
    ) {
        let window = Window::new(&self.event_loop).expect("Couldn't create window.");
        let window_id = window.id();

        let context = Context::new(window).await;
        let redraw = setup(&context);

        self.contexts.insert(window_id, context);
        self.redraws.insert(window_id, Box::new(redraw));
    }
}

impl Default for Studio {
    fn default() -> Self {
        Studio {
            event_loop: EventLoop::new(),
            contexts: HashMap::new(),
            redraws: HashMap::new(),
        }
    }
}
