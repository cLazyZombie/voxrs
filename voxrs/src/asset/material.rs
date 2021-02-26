use serde::Deserialize;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, TextureAsset,
};
use crate::io::FileSystem;

#[derive(Asset)]
pub struct MaterialAsset {
    pub diffuse_tex: AssetHandle<TextureAsset>,
}

#[derive(Deserialize)]
struct MaterialAssetRaw {
    diffuse_tex: String,
}

impl MaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: MaterialAssetRaw = serde_json::from_str(s).unwrap();

        let diffuse_tex = asset_manager.get::<TextureAsset>(&raw.diffuse_tex.into());

        Self { diffuse_tex }
    }
}
