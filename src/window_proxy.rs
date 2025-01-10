use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Fullscreen, Window},
};

use crate::gpu::GpuState;

/// Proxy is intended to build (asyncronously on **WASM** and syncronously on **Native Platforms**) and send the gpu state with the window (that is stored in the GpuState) to the main event loop.
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
