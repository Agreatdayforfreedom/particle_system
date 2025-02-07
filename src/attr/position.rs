#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Position {
    // this can be named 'center' and the struct itself can hold more paramenters like radius or area, if it's a circle or a square.
    pub position: [f32; 4],
}

impl Default for Position {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
