use core::f32;
use std::borrow::Cow;

use crate::attr::{AttrContext, ShaderBuilder};
use crate::uniform;
use crate::window::InputEvent;
use crate::{
    camera::{Camera2D, Camera2DUniform, Camera3D, Camera3DUniform, CameraController},
    quad::{Quad, VERTICES},
    uniform::Uniform,
};

use cgmath::{InnerSpace, Vector3};
use naga_oil::compose::{ComposableModuleDescriptor, Composer, NagaModuleDescriptor};
use rand::Rng;
use wgpu::util::DeviceExt;
use wgpu::QuerySet;

#[cfg(target_arch = "wasm32")]
const PARTICLE_POOLING: u64 = 250_000;

#[cfg(not(target_arch = "wasm32"))]
const PARTICLE_POOLING: u64 = 1_000_000;

//*  4_194_240 / 64 = 65535 MAX (x) DISPATCHES
// #[cfg(not(target_arch = "wasm32"))]
// const PARTICLE_POOLING: u64 = 4_194_241;

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
        let mut rng = rand::thread_rng();
        // pos
        chunk[0] = 0.1;
        chunk[1] = 0.1;
        chunk[2] = 0.1;
        chunk[3] = 0.0;
        //dir
        chunk[4] = rng.gen_range(0.01..0.05);
        chunk[5] = rng.gen_range(0.01..0.05);
        chunk[6] = rng.gen_range(0.01..0.05);
        //velocity
        chunk[7] = rng.gen_range(-0.1..0.1);

        //lifetime
        // chunk[8] = 0.0;
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
    pub particle_uniform: uniform::Uniform<AttrContext>,
    /**test */
    /// contains all the data to compute the paricles. \
    /// holds the *particles buffer* at **@binding(0)** \
    /// holds the *simulation params buffer* at **@binding(1)** \
    /// holds the *delta time buffer* at **@binding(2)**
    bind_group: wgpu::BindGroup,
    time: f64,
}

impl System {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let mut camera = Camera3D::new(Uniform::<Camera3DUniform>::new(&device));

        let module = ShaderBuilder::build_module(&include_str!("shaders/vfx_render.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            label: Some("vfx_render.wgsl"),
        });

        let module =
            ShaderBuilder::build_module(&include_str!("shaders/vfx_compute.wgsl").replace(
                ";;COMPUTE_CODE",
                "var a = 20.0;			
        var b = 16.0 / 3.0;
        var c = 38.0;

        let distance = sqrt(
            particle.position.x * particle.position.x + 
            particle.position.y * particle.position.y + 
            particle.position.z * particle.position.z
        );

        let max_distance = sqrt(
            100.0*100.0+
            100.0*100.0+
            100.0*100.0
        );
    
        let float = clamp(0.0, 1.0, f32(distance / max_distance));

        var dx = a * (particle.position.y - particle.position.x);
        var dy = particle.position.x * (c - particle.position.z) - particle.position.y;
        var dz = particle.position.x * particle.position.y - b * particle.position.z;
        dx *= particle_uniform.velocity;
        dy *= particle_uniform.velocity;
        dz *= particle_uniform.velocity;
        particle.position.x +=  dx * particle.dir.x * uniforms.delta_time;
        particle.position.y +=  dy * particle.dir.y * uniforms.delta_time;
        particle.position.z +=  dz * particle.dir.z * uniforms.delta_time;
        particle.position.w =  float;
",
            ));
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            label: Some("vfx_compute.wgsl"),
        });

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
        // for testing (pos.xyz, mass)

        let mut uniform_bytes: Vec<f32> = vec![0.0];

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Compute Buffer"),
            contents: bytemuck::cast_slice(&uniform_bytes),
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
        let particle_uniform = Uniform::<AttrContext>::new(&device);
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    &create_compute_bind_group_layout(device),
                    &particle_uniform.bind_group_layout,
                ],
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
            particle_uniform,
            // camera_pos_uniform: Uniform::<f32>::new(&device),
            vertex_buffer,
            pipeline,
            compute_pipeline,
            time: 0.0,
        }
    }

    pub fn input(&mut self, event: InputEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: instant::Duration) {
        self.time += dt.as_secs_f64();
        self.camera_controller.update_camera(&mut self.camera);
        self.camera.build_view_projection_matrix();
        self.camera.update((0.0, 0.0, 0.0).into());

        let uniform_bytes: Vec<f32> = vec![dt.as_secs_f32()];

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&uniform_bytes),
        );
        self.particle_uniform.write(queue);

        self.camera.uniform.write(queue);
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        query_render_timing: &QuerySet,
        query_update_timing: &QuerySet,
    ) {
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: query_update_timing,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_bind_group(1, &self.particle_uniform.bind_group, &[]);
            cpass.dispatch_workgroups((PARTICLE_POOLING as f32 / 64.0).ceil() as u32, 1, 1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                    query_set: query_render_timing,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.camera.uniform.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.particle_buffer.slice(..));
            rpass.draw(0..6, 0..PARTICLE_POOLING as u32);
        }
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
                            format: wgpu::VertexFormat::Float32x4,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraPosition {
    position: [f32; 3],
}

impl Default for CameraPosition {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
        }
    }
}
