use crate::texture::{self, create_bind_group_texture_layout};

pub struct Bloom {
    brightness_target_texture: texture::Texture,
    horizontal_blur_target_texture: texture::Texture,
    vertical_blur_target_texture: texture::Texture,
    final_target_texture: texture::Texture,
    brightness_pipeline: wgpu::RenderPipeline,
    horizontal_blur_pipeline: wgpu::RenderPipeline,
    vertical_blur_pipeline: wgpu::RenderPipeline,
    blend_pipeline: wgpu::RenderPipeline,
}

impl Bloom {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, size: (u32, u32)) -> Self {
        let brightness_target_texture = texture::Texture::empty(&device, size, Some("offscreen"))
            .expect("Failed to build empty texture");

        let horizontal_blur_target_texture =
            texture::Texture::empty(&device, size, Some("offscreen"))
                .expect("Failed to build empty texture");

        let vertical_blur_target_texture =
            texture::Texture::empty(&device, size, Some("offscreen"))
                .expect("Failed to build empty texture");

        let final_target_texture = texture::Texture::empty(&device, size, Some("offscreen"))
            .expect("Failed to build empty texture");

        let bind_group_layout = create_bind_group_texture_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let final_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
            });
        let shader_fullscreen_quad = device.create_shader_module(wgpu::include_wgsl!(
            "../shaders/fullscreen_quad_vertex.wgsl"
        ));
        let shader_brightness =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/brightness.wgsl"));
        let shader_blend =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/bloom_blend.wgsl"));
        let shader_blur = device.create_shader_module(wgpu::include_wgsl!("../shaders/blur.wgsl"));
        let brightness_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brightness pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_fullscreen_quad,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_brightness,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertical_blur_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Verical blur pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_fullscreen_quad,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_blur,
                    entry_point: Some("vertical_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let horizontal_blur_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Horizontal blur pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_fullscreen_quad,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_blur,
                    entry_point: Some("horizontal_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let blend_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brightness pipeline"),
            layout: Some(&final_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_fullscreen_quad,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_blend,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            brightness_pipeline,
            horizontal_blur_pipeline,
            vertical_blur_pipeline,
            blend_pipeline,

            horizontal_blur_target_texture,
            vertical_blur_target_texture,
            brightness_target_texture,
            final_target_texture,
        }
    }

    pub fn get_final_texture(&self) -> &texture::Texture {
        &self.final_target_texture
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, scene_texture: &texture::Texture) {
        let mut brightness_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.brightness_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        brightness_rpass.set_pipeline(&self.brightness_pipeline);
        brightness_rpass.set_bind_group(0, &scene_texture.bind_group, &[]);
        brightness_rpass.draw(0..6, 0..1);

        drop(brightness_rpass);

        let mut vertical_blur_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.vertical_blur_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        vertical_blur_rpass.set_pipeline(&self.vertical_blur_pipeline);
        vertical_blur_rpass.set_bind_group(0, &self.brightness_target_texture.bind_group, &[]);
        vertical_blur_rpass.draw(0..6, 0..1);

        drop(vertical_blur_rpass);
        let mut horizontal_blur_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.horizontal_blur_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        horizontal_blur_rpass.set_pipeline(&self.horizontal_blur_pipeline);
        horizontal_blur_rpass.set_bind_group(0, &self.vertical_blur_target_texture.bind_group, &[]);
        horizontal_blur_rpass.draw(0..6, 0..1);

        drop(horizontal_blur_rpass);

        let mut final_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.final_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        final_rpass.set_pipeline(&self.blend_pipeline);
        final_rpass.set_bind_group(0, &self.horizontal_blur_target_texture.bind_group, &[]);
        final_rpass.set_bind_group(1, &scene_texture.bind_group, &[]);
        final_rpass.draw(0..6, 0..1);
    }
}
