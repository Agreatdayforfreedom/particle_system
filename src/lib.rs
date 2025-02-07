mod attr;
mod camera;
mod egui;
mod gpu;
mod quad;
mod system;
mod uniform;
mod window;
mod window_proxy;

use gpu::GpuState;
use window::App;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let event_loop = EventLoop::<GpuState>::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(&event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run_app(&mut app).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }
}
