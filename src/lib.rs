mod camera;
mod gpu;
mod quad;
mod system;
mod uniform;
mod window;

use window::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
