use serde::Deserialize;
use voxrs_types::io::FileSystem;

use crate::ShaderAsset;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, TextureAsset,
};

#[derive(Asset)]
pub struct MaterialAsset {
    pub diffuse_tex: AssetHandle<TextureAsset>,
    pub shader: AssetHandle<ShaderAsset>,
}

#[derive(Deserialize)]
struct MaterialAssetRaw {
    diffuse_tex: String,
    shader: String,
}

impl MaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: MaterialAssetRaw = serde_json::from_str(s).unwrap();

        let diffuse_tex = asset_manager.get::<TextureAsset>(&raw.diffuse_tex.into());
        let shader = asset_manager.get::<ShaderAsset>(&raw.shader.into());

        Self {
            diffuse_tex,
            shader,
        }
    }
}
