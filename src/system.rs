use core::f32;

use crate::{
    camera::{Camera2D, Camera2DUniform, Camera3D, Camera3DUniform, CameraController},
    quad::{Quad, VERTICES},
    uniform::Uniform,
};
use cgmath::{InnerSpace, Vector3};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;

const PARTICLE_POOLING: u64 = 2000000;

fn dv() -> Vector3<f32> {
    let mut rng = rand::thread_rng();

    let theta = rng.gen_range(0.0..2.0 * f32::consts::PI);
    let phi = rng.gen_range(0.0..f32::consts::PI);

    let x = phi.sin() * theta.cos();
    let y = phi.sin() * theta.sin();
    let z = phi.cos();

    cgmath::Vector3::new(x, y, z).normalize()
}

fn generate_particles() -> Vec<f32> {
    let mut particles = vec![0.0f32; 8 * PARTICLE_POOLING as usize];

    for chunk in particles.chunks_mut(8) {
        // pos
        chunk[0] = 0.0;
        chunk[1] = 0.0;
        chunk[2] = 0.0;
        //velocity
        chunk[3] = 0.0;
        let dir = dv();
        //dir
        chunk[4] = dir.x;
        chunk[5] = dir.y;
        chunk[6] = dir.z;

        //lifetime
        chunk[7] = 0.0;
    }
    particles
}

pub struct System {
    camera: Camera3D,
    camera_controller: CameraController,
    pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,

    vertex_buffer: wgpu::Buffer,
    simulation_buffer: wgpu::Buffer,
    particle_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    camera_pos_uniform: Uniform<f32>,

    /// contains all the data to compute the paricles. \
    /// holds the *particles buffer* at **@binding(0)** \
    /// holds the *simulation params buffer* at **@binding(1)** \
    /// holds the *delta time buffer* at **@binding(2)**
    bind_group: wgpu::BindGroup,
}

impl System {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let mut camera = Camera3D::new(Uniform::<Camera3DUniform>::new(&device));

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader.wgsl"));
        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/vfx_compute.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main_pipeline_layout"),
            bind_group_layouts: &[&camera.uniform.bind_group_layout],
            push_constant_ranges: &[],
        });
        camera.build_view_projection_matrix();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // compute
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer"),
            contents: bytemuck::cast_slice(&generate_particles()),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });
        // let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some(&format!("Particle Buffer")),
        //     size: 8 * 4 * PARTICLE_POOLING,
        //     usage: wgpu::BufferUsages::VERTEX
        //         | wgpu::BufferUsages::STORAGE
        //         | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        let simulation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Compute Buffer"),
            contents: bytemuck::bytes_of(&[0.0, 0.0]), //dummy data
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Compute Buffer"),
            contents: bytemuck::bytes_of(&[0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &create_compute_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: simulation_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let pipeline = create_render_pipeline(device, &shader, config.format, &pipeline_layout);
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&create_compute_bind_group_layout(device)],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particles compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("simulate"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            camera,
            camera_controller: CameraController::new(2.0),
            bind_group,
            particle_buffer,
            simulation_buffer,
            uniform_buffer,
            camera_pos_uniform: Uniform::<f32>::new(&device),
            vertex_buffer,
            pipeline,
            compute_pipeline,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }
    pub fn update(&mut self, queue: &wgpu::Queue, dt: instant::Duration) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[dt.as_secs_f32()]),
        );

        self.camera_controller.update_camera(&mut self.camera);
        self.camera.build_view_projection_matrix();
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
            let mut rpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.compute_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.dispatch_workgroups((PARTICLE_POOLING as f32 / 64.0).ceil() as u32, 1, 1);
        }
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

            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.particle_buffer.slice(..));

            rpass.draw(0..6, 0..PARTICLE_POOLING as u32);
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
            buffers: &[
                Quad::desc(),
                wgpu::VertexBufferLayout {
                    array_stride: 8 * 4,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        //position
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 1,
                        },
                        // wgpu::VertexAttribute { // dir?
                        //     format: wgpu::VertexFormat::Float32x2,
                        //     offset: 8,
                        //     shader_location: 1,
                        // },
                    ],
                },
            ],
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
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
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

fn create_compute_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Particle Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}
