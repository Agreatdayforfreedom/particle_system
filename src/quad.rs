#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Quad {
    position: [f32; 2],
}

pub const VERTICES: &[Quad] = &[
    Quad {
        position: [0.0, 1.0],
    },
    Quad {
        position: [1.0, 0.0],
    },
    Quad {
        position: [0.0, 0.0],
    },
    Quad {
        position: [0.0, 1.0],
    },
    Quad {
        position: [1.0, 1.0],
    },
    Quad {
        position: [1.0, 0.0],
    },
];

impl Quad {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
