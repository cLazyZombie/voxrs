use serde::Deserialize;

use crate::io::FileSystem;
use super::{AssetHandle, AssetManager, TextureAsset, assets::{Asset, AssetType}};


pub struct MaterialAsset {
    pub diffuse_tex: AssetHandle<TextureAsset>,
}

#[derive(Deserialize)]
struct MaterialAssetRaw {
    diffuse_tex: String,
}

impl Asset for MaterialAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Material
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }
}

impl MaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: MaterialAssetRaw = serde_json::from_str(s).unwrap();

        let diffuse_tex = asset_manager.get::<TextureAsset>(&raw.diffuse_tex.into());

        Self { diffuse_tex }
    }
}