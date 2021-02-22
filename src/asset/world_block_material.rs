use std::collections::HashMap;

use crate::io::FileSystem;

use super::{AssetHandle, AssetManager, AssetPath, MaterialAsset, assets::{Asset, AssetType}};
use serde::Deserialize;

pub struct WorldBlockMaterialAsset {
    pub material_handles: HashMap<u8, AssetHandle<MaterialAsset>>,
}

impl Asset for WorldBlockMaterialAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::WorldBlockMaterial
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }

    fn need_build(&self) -> bool {
        false
    }

    fn build(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}
}

impl WorldBlockMaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldBlockMaterialAssetRaw = serde_json::from_str(s).unwrap();

        let mut material_handles = HashMap::new();
        for entity in &raw.materials {
            let material = asset_manager.get::<MaterialAsset>(&AssetPath::from_str(&entity.material));
            material_handles.insert(entity.id, material);
        }

        Self { material_handles }
    }
}

#[derive(Deserialize)]
struct WorldBlockMaterialEntity {
    pub id: u8,
    pub material: String,
}

#[derive(Deserialize)]
struct WorldBlockMaterialAssetRaw {
    pub materials: Vec<WorldBlockMaterialEntity>,
}
