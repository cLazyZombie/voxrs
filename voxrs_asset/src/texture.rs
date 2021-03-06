use voxrs_rhi::Texture;

use crate::handle::AssetLoadError;

use super::{
    assets::{Asset, AssetType},
    AssetBuildResult,
};

#[derive(Asset)]
pub struct TextureAsset {
    pub buf: Vec<u8>,
    pub texture: AssetBuildResult<Texture>,
}

impl TextureAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            texture: AssetBuildResult::NotBuilt,
        }
    }
}

impl TextureAsset {
    async fn load_asset<F: voxrs_types::io::FileSystem>(
        path: &crate::AssetPath,
        _manager: &mut crate::AssetManager<F>,
        device: Option<&wgpu::Device>,
        queue: Option<&wgpu::Queue>,
    ) -> Result<Self, crate::handle::AssetLoadError> {
        let result;
        if let Ok(v) = F::read_binary(path).await {
            let mut texture = TextureAsset::new(v);
            if let (Some(device), Some(queue)) = (device, queue) {
                texture.build(device, queue);
            }
            result = Ok(texture);
        } else {
            result = Err(AssetLoadError::Failed);
        }
        result
    }

    fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        match &self.texture {
            AssetBuildResult::Ok(_) => {
                log::warn!("texture already built");
            }
            AssetBuildResult::Err(err) => {
                log::warn!("texture build already has error. {:?}", err);
            }
            AssetBuildResult::NotBuilt => {
                let result = Texture::from_bytes(device, queue, &self.buf, "texture");
                match result {
                    Ok(texture) => {
                        self.texture = AssetBuildResult::Ok(texture);
                    }
                    Err(err) => {
                        log::error!("texture build error. err: {}", &err.to_string());
                        self.texture = AssetBuildResult::Err(err.context("texture build error"));
                    }
                }
            }
        }
    }
}
