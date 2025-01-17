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
    system: System,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        println!("w:{}, h: {}", size.width, size.height);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();
        println!("{:?}", adapter.features());
        let (device, mut queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

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
            width: size.width,
            height: size.height,
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
        }
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
