#[allow(unused_imports)]
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowId},
};

use crate::gpu::GpuState;

pub struct Proxy {
    proxy: Option<EventLoopProxy<GpuState>>,
}

impl Proxy {
    pub fn new(proxy: EventLoopProxy<GpuState>) -> Self {
        Self { proxy: Some(proxy) }
    }

    pub fn send(&mut self, event_loop: &ActiveEventLoop) {
        let Some(proxy) = self.proxy.take() else {
            log::info!("trying to send a proxy event");
            return;
        };

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
            }
        }
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let mut size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| {
                    let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
                    let height = win.inner_height().unwrap().as_f64().unwrap() as u32;
                    let factor = window.scale_factor();
                    let logical = LogicalSize { width, height };

                    let PhysicalSize { width, height }: PhysicalSize<u32> =
                        logical.to_physical(factor);

                    size = PhysicalSize::new(width, height);

                    log::info!("window size configured from web_sys window: {:?}", size);

                    win.document()
                })
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("main").unwrap();
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }
        #[cfg(not(target_arch = "wasm32"))]
        window.set_fullscreen(Some(Fullscreen::Borderless(None)));

        #[cfg(target_arch = "wasm32")]
        {
            let gpu = GpuState::new(window, size);
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = gpu.await;
                assert!(proxy.send_event(gpu).is_ok());
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use pollster::FutureExt;
            assert!(proxy
                .send_event(GpuState::new(window, size).block_on())
                .is_ok());
        }
    }
}

enum GpuStage {
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Ready(state) = &mut self.state else {
            log::info!("Reciving window_event but the gpu is not initialized");
            return;
        };

        if !state.input(&event) {
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
                    state.render();
                }

                _ => (),
            }
        }
    }
}
