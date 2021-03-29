use std::borrow::Cow;

use wgpu::{ShaderFlags, ShaderModuleDescriptor};

use super::{
    assets::{Asset, AssetType},
    AssetBuildResult,
};

#[derive(Asset)]
pub struct ShaderAsset {
    pub buf: String,
    pub module: AssetBuildResult<wgpu::ShaderModule>,
}

impl ShaderAsset {
    pub fn new(buf: String) -> Self {
        Self {
            buf,
            module: AssetBuildResult::NotBuilt,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) {
        let shader_source = wgpu::ShaderSource::Wgsl(Cow::from(&self.buf));

        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: shader_source,
            flags: ShaderFlags::VALIDATION | wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
        });
        self.module = AssetBuildResult::Ok(module);
    }
}
