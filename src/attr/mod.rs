pub mod position;
pub mod velocity;

pub use position::*;
pub use velocity::*;

use naga_oil::compose::{Composer, NagaModuleDescriptor};
use wgpu::naga::Module;

use crate::uniform::Uniform;

pub struct ShaderBuilder {
    main_code: String,
}

impl ShaderBuilder {
    pub fn build_module(source: &str) -> Module {
        let mut composer = Composer::default();
        let module = match composer.make_naga_module(NagaModuleDescriptor {
            source,
            file_path: "./shaders/vfx_render.wgsl",
            shader_defs: [(Default::default())].into(),
            ..Default::default()
        }) {
            Ok(module) => Ok(module),
            Err(e) => {
                println!("error: {e:#?}");
                Err(e)
            }
        };
        let module = module.unwrap();
        module
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct AttrContext {
    pub position: Position,
    pub velocity: Velocity,
    _pad: [f32; 3],
}

impl AttrContext {
    pub fn update_uniform(&mut self, position: [f32; 4], velocity: f32) {
        self.position.position = position;
        self.velocity.vel += velocity;
    }
}

impl Default for AttrContext {
    fn default() -> Self {
        Self {
            position: Position::default(),
            velocity: Velocity::default(),
            _pad: [0.0, 0.0, 0.0],
        }
    }
}
