use std::collections::HashMap;

use crate::io::FileSystem;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, MaterialAsset,
};
use serde::Deserialize;

#[derive(Asset)]
pub struct WorldMaterialAsset {
    pub material_handles: HashMap<u8, AssetHandle<MaterialAsset>>,
}

impl WorldMaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldMaterialAssetRaw = serde_json::from_str(s).unwrap();

        let mut material_handles = HashMap::new();
        for entity in &raw.materials {
            let material =
                asset_manager.get::<MaterialAsset>(&AssetPath::from_str(&entity.material));
            material_handles.insert(entity.id, material);
        }

        Self { material_handles }
    }
}

#[derive(Deserialize)]
struct WorldMaterialEntity {
    pub id: u8,
    pub material: String,
}

#[derive(Deserialize)]
struct WorldMaterialAssetRaw {
    pub materials: Vec<WorldMaterialEntity>,
}
