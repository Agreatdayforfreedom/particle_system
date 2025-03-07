use std::num::NonZeroU32;

use bytemuck::Contiguous;
use egui::{Align2, Pos2, Rect};
use egui_wgpu::ScreenDescriptor;
use wgpu::core::device;
use wgpu::{SurfaceTexture, TextureFormat};
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::egui::EguiRenderer;
use crate::profiler::{self, Profiler, QueryTimestampPass};
use crate::system::System;
use crate::window::InputEvent;

#[cfg(target_arch = "wasm32")]
type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
type Rc<T> = std::sync::Arc<T>;

#[allow(dead_code)]
pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Rc<Window>,
    egui: EguiRenderer,
    system: System,
    profiler: Profiler,
}

impl GpuState {
    pub async fn new(window: Window, size: PhysicalSize<u32>) -> Self {
        let window = Rc::new(window);

        #[cfg(not(target_arch = "wasm32"))]
        let size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        let size = size;

        println!("w:{}, h: {}", size.width, size.height);

        #[cfg(target_arch = "wasm32")]
        {
            log::info!("Initial window size: {:?}", size);
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(Rc::clone(&window)).unwrap();

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
                    required_features: wgpu::Features::default()
                        | wgpu::Features::TIMESTAMP_QUERY
                        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
                        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
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

        log::info!("surface caps: {:?}", &surface_caps);
        log::info!("surface format: {:?}", &surface_format);
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
        let egui = EguiRenderer::new(&device, config.format, None, 1, window.as_ref());

        let mut profiler = Profiler::default();

        profiler.add_query_timestamp_pass(QueryTimestampPass::new(Some("Render"), &device));
        profiler.add_query_timestamp_pass(QueryTimestampPass::new(Some("Compute"), &device));
        Self {
            surface,
            device,
            queue,
            config,
            system,
            egui,
            window,
            profiler,
        }
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    pub fn input(&mut self, event: InputEvent) {
        if let InputEvent::Window(event) = event {
            if self.egui.handle_input(self.window.as_ref(), event) {
                return;
            }
        }
        self.system.input(event);
    }
    pub fn update(&mut self, dt: instant::Duration) {
        self.system.update(&self.queue, dt);

        // println!("FPS: {}", 1.0 / dt.as_secs_f64());
    }
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        self.config.width = width.into();
        self.config.height = height.into();

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, dt: instant::Duration) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.system
            .render(&mut encoder, &context_view, &self.profiler);

        for query in &mut self.profiler.timestamps {
            query.resolve(&mut encoder);
        }

        {
            self.egui.begin_frame(&self.window);

            egui::Window::new("DEBUG")
                .default_open(false)
                .vscroll(true)
                .resizable(false)
                .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
                .default_size([200.0, 75.0])
                .show(self.egui.context(), |ui| {
                    ui.label(format!("FPS: {}", (1.0 / dt.as_secs_f64())));
                    //TODO
                    ui.label(format!(
                        "Render time: {}µs",
                        self.profiler.timestamps.get(0).unwrap().pass_time
                    ));
                    ui.label(format!(
                        "Update time: {}µs",
                        self.profiler.timestamps.get(1).unwrap().pass_time
                    ));

                    let _ = ui.add(egui::Slider::new(
                        &mut self.system.particle_uniform.data.velocity.vel,
                        -100.0..=100.0,
                    ));
                });

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: 1.0,
            };

            self.egui.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &context_view,
                screen_descriptor,
            );
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        for query in &mut self.profiler.timestamps {
            query.map();

            self.device
                .poll(wgpu::MaintainBase::wait())
                .panic_on_timeout();

            query.unmap();
        }
    }
}
