use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::gpu::GpuState;

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<GpuState>,
    time: instant::Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            time: instant::Instant::now(),
            state: None,
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        self.time = instant::Instant::now();
        let state = GpuState::new(Arc::clone(&window));

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            panic!("No Window state found.")
        };

        if !state.input(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    let now = instant::Instant::now();
                    let dt = now - self.time;
                    self.time = now;
                    self.window.as_ref().unwrap().request_redraw();
                    state.update(dt);
                    state.render();
                }

                _ => (),
            }
        }
    }
}
