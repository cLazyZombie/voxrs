use wgpu::{util::make_spirv, ShaderFlags, ShaderModuleDescriptor};

use super::{
    assets::{Asset, AssetType},
    AssetBuildResult,
};

#[derive(Asset)]
pub struct ShaderAsset {
    pub buf: Vec<u8>,
    pub module: AssetBuildResult<wgpu::ShaderModule>,
}

impl ShaderAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            module: AssetBuildResult::NotBuilt,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) {
        let shader_source = make_spirv(&self.buf);
        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: shader_source,
            flags: ShaderFlags::VALIDATION,
        });
        self.module = AssetBuildResult::Ok(module);
    }
}
