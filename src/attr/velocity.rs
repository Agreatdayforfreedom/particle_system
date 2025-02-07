#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Velocity {
    pub vel: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { vel: 0.0 }
    }
}
