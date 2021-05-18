use wgpu::{util::make_spirv, ShaderFlags, ShaderModuleDescriptor};

use crate::handle::AssetLoadError;

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

    async fn load_asset<F: voxrs_types::io::FileSystem>(
        path: &crate::AssetPath,
        _manager: &mut crate::AssetManager<F>,
        device: Option<&wgpu::Device>,
        queue: Option<&wgpu::Queue>,
    ) -> Result<Self, crate::handle::AssetLoadError>
    where
        Self: Sized,
    {
        let result;
        if let Ok(v) = F::read_binary(path).await {
            let mut shader = ShaderAsset::new(v);
            if let (Some(device), Some(queue)) = (device, queue) {
                shader.build(device, queue);
            }
            result = Ok(shader);
        } else {
            result = Err(AssetLoadError::Failed);
        }
        result
    }

    fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) {
        let shader_source = make_spirv(&self.buf);
        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: shader_source,
            flags: ShaderFlags::VALIDATION,
        });
        self.module = AssetBuildResult::Ok(module);
    }
}
