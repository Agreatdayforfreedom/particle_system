use std::num::NonZeroU32;

use bytemuck::Contiguous;
use egui::{Align2, Pos2, Rect};
use egui_wgpu::ScreenDescriptor;
use wgpu::core::device;
use wgpu::{SurfaceTexture, TextureFormat};
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::egui::EguiRenderer;
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

    //timing
    gpu_render_time: f64,
    query_render_timing: wgpu::QuerySet,
    query_render_resolve_buffer: wgpu::Buffer,
    query_render_result_buffer: wgpu::Buffer,

    gpu_update_time: f64,
    query_update_timing: wgpu::QuerySet,
    query_update_resolve_buffer: wgpu::Buffer,
    query_update_result_buffer: wgpu::Buffer,
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

        let query_render_timing = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Query set render timing"),
            count: 2,
            ty: wgpu::QueryType::Timestamp,
        });

        let query_render_resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query set buffer render timing"),
            size: 2 * 8,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_render_result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query resolve buffer render result"),
            size: query_render_resolve_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let query_update_timing = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Query set update timing"),
            count: 2,
            ty: wgpu::QueryType::Timestamp,
        });

        let query_update_resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query set buffer update timing"),
            size: 2 * 8,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_update_result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query resolve buffer update result"),
            size: query_render_resolve_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            system,
            egui,
            window,

            gpu_render_time: 0.0,
            gpu_update_time: 0.0,
            query_render_resolve_buffer,
            query_render_result_buffer,
            query_update_resolve_buffer,
            query_update_result_buffer,
            query_render_timing,
            query_update_timing,
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

        self.system.render(
            &mut encoder,
            &context_view,
            &self.query_render_timing,
            &self.query_update_timing,
        );

        encoder.resolve_query_set(
            &self.query_render_timing,
            0..2,
            &self.query_render_resolve_buffer,
            0,
        );

        encoder.copy_buffer_to_buffer(
            &self.query_render_resolve_buffer,
            0,
            &self.query_render_result_buffer,
            0,
            self.query_render_result_buffer.size(),
        );

        encoder.resolve_query_set(
            &self.query_update_timing,
            0..2,
            &self.query_update_resolve_buffer,
            0,
        );

        encoder.copy_buffer_to_buffer(
            &self.query_update_resolve_buffer,
            0,
            &self.query_update_result_buffer,
            0,
            self.query_update_result_buffer.size(),
        );

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
                    ui.label(format!("Render time: {}µs", self.gpu_render_time));
                    ui.label(format!("Update time: {}µs", self.gpu_update_time));

                    let _ = ui.add(egui::Slider::new(
                        &mut self.system.particle_uniform.data.velocity.vel,
                        0.0..=100.0,
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

        // let (sender, receiver) = flume::bounded(1);

        let _ = self
            .query_render_result_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| {});

        let _ = self
            .query_update_result_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| {});

        self.device
            .poll(wgpu::MaintainBase::wait())
            .panic_on_timeout();
        {
            let slice: &[u8] = &self.query_render_result_buffer.slice(..).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&slice);

            self.gpu_render_time = (timestamps[1].wrapping_sub(timestamps[0]) / 1000) as f64;
        }
        self.query_render_result_buffer.unmap();

        {
            let slice: &[u8] = &self.query_update_result_buffer.slice(..).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&slice);

            self.gpu_update_time = (timestamps[1].wrapping_sub(timestamps[0]) / 1000) as f64;
        }
        self.query_update_result_buffer.unmap();
    }
}
