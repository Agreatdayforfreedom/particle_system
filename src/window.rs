#[allow(unused_imports)]
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowId},
};

use crate::{gpu::GpuState, window_proxy::Proxy};

pub enum InputEvent<'a> {
    Window(&'a WindowEvent),
    Decive(&'a DeviceEvent),
}

pub enum GpuStage {
    Wait(Proxy),
    Ready(GpuState),
}
use self::GpuStage::*;

pub struct App {
    state: GpuStage,
    time: instant::Instant,
}

impl App {
    pub fn new(event_loop: &EventLoop<GpuState>) -> Self {
        Self {
            time: instant::Instant::now(),
            state: GpuStage::Wait(Proxy::new(event_loop.create_proxy())),
        }
    }
}

impl ApplicationHandler<GpuState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Wait(proxy) = &mut self.state {
            proxy.send(event_loop);
        }
        self.time = instant::Instant::now();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, gpu: GpuState) {
        log::info!("Gpu initialized correctly!");
        self.state = Ready(gpu);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Ready(state) = &mut self.state else {
            log::info!("Reciving device_event but the gpu is not initialized");
            return;
        };

        state.input(InputEvent::Decive(&event));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Ready(state) = &mut self.state else {
            log::info!("Reciving window_event but the gpu is not initialized");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                log::info!("Resizing: {:?}", new_size);
                state.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                let now = instant::Instant::now();
                let dt = now - self.time;
                self.time = now;
                state.window().request_redraw();
                state.update(dt);
                state.render(dt);
            }

            _ => {
                state.input(InputEvent::Window(&event));
            }
        }
    }
}
