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
    pub vertex_shader: AssetHandle<ShaderAsset>,
    pub frag_shader: AssetHandle<ShaderAsset>,
}

#[derive(Deserialize)]
struct MaterialAssetRaw {
    diffuse_tex: String,
    vertex_shader: String,
    frag_shader: String,
}

impl MaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: MaterialAssetRaw = serde_json::from_str(s).unwrap();

        let diffuse_tex = asset_manager.get::<TextureAsset>(&raw.diffuse_tex.into());
        let vertex_shader = asset_manager.get::<ShaderAsset>(&raw.vertex_shader.into());
        let frag_shader = asset_manager.get::<ShaderAsset>(&raw.frag_shader.into());

        Self {
            diffuse_tex,
            vertex_shader,
            frag_shader,
        }
    }
}
