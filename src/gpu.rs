use pollster::block_on;
use std::sync::Arc;
use winit::{event::*, window::Window};

use crate::system::System;

#[allow(dead_code)]
pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    system: System,
}

impl GpuState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        println!("w:{}, h: {}", size.width, size.height);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        println!("{:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let limits = device.limits();
        #[cfg(target_arch = "wasm32")]
        log::info!("{:?}", limits);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(800),
            height: size.height.max(600), // setting this because Fullscreen does not work on web: https://developer.mozilla.org/en-US/docs/Glossary/Transient_activation
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let system = System::new(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            system,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // false
        self.system.input(event)
    }
    pub fn update(&mut self, dt: instant::Duration) {
        self.system.update(&self.queue, dt);
        // println!("FPS: {}", 1.0 / dt.as_secs_f64());
    }
    pub fn render(&mut self) {
        self.system.render(&self.device, &self.queue, &self.surface);
    }
}
