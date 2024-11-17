// this is a 2d camera
use cgmath::{
    Array, EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Transform, Vector2, Vector3,
};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0,
    0.0, 0.0, 0.0, 1.0,
);
use crate::uniform::Uniform;

pub struct Camera {
    pub position: Vector3<f32>,
    pub scale: Vector2<f32>,
    pub uniform: Uniform<CameraUniform>,
}

impl Camera {
    pub fn new(uniform: Uniform<CameraUniform>) -> Self {
        Self {
            uniform,
            scale: (WIDTH, HEIGHT).into(),
            position: (0.0, 0.0, 0.0).into(),
        }
    }
    pub fn update(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.uniform.data.update(position);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub proj: [[f32; 4]; 4],
}
impl CameraUniform {
    fn update(&mut self, position: Vector3<f32>) {
        let view = Matrix4::from_translation(-position);
        let ortho = OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -WIDTH / 2.0,
                WIDTH / 2.0,
                -HEIGHT / 2.0,
                HEIGHT / 2.0,
                -50.0,
                50.0,
            );
        self.proj = (ortho * view).into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        let position = Vector3::new(0.0, 0.0, 1.0);

        let view = Matrix4::from_translation(-position);
        let ortho = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0);
        Self {
            proj: (ortho * view).into(),
        }
    }
}
