use wgpu::util::DeviceExt;

pub struct Uniform<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl<T> Uniform<T>
where
    T: bytemuck::Pod + bytemuck::Zeroable + Default,
{
    pub fn new(device: &wgpu::Device) -> Self {
        let data = T::default();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Uniform Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        Self {
            data,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}
