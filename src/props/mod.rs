pub mod position;
pub mod velocity;

pub use position::*;
pub use velocity::*;

use naga_oil::compose::{Composer, NagaModuleDescriptor};
use wgpu::naga::Module;

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
