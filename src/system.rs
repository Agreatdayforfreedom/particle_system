use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraUniform},
    quad::{Quad, VERTICES},
    uniform::Uniform,
};

pub struct System {
    camera: Camera,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
}

impl System {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let camera = Camera::new(Uniform::<CameraUniform>::new(&device));

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main_pipeline_layout"),
            bind_group_layouts: &[&camera.uniform.bind_group_layout],
            push_constant_ranges: &[],
        });

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline = create_render_pipeline(device, &shader, config.format, &pipeline_layout);

        Self {
            camera,

            pipeline,
            buffer: quad_buffer,
        }
    }

    pub fn input() {}
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.camera.update((0.0, 0.0, 0.0).into());
        self.camera.uniform.write(queue);
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.camera.uniform.bind_group, &[]);

            rpass.set_vertex_buffer(0, self.buffer.slice(..));

            rpass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));

        frame.present();
    }
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Quad::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
