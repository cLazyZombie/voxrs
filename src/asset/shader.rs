use wgpu::util::make_spirv;

use super::{AssetBuildResult, assets::{Asset, AssetType}};


pub struct ShaderAsset {
    pub buf: Vec<u8>,
    pub module: AssetBuildResult<wgpu::ShaderModule>,
}

impl Asset for ShaderAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Shader
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }
}

impl ShaderAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            module: AssetBuildResult::NotBuilt,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) {
        let module = device.create_shader_module(make_spirv(&self.buf));
        self.module = AssetBuildResult::Ok(module);
    }
}