use std::collections::HashMap;

use crate::handle::AssetLoadError;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, MaterialAsset,
};
use serde::Deserialize;
use voxrs_types::io::FileSystem;

#[derive(Asset)]
pub struct WorldMaterialAsset {
    pub material_handles: HashMap<u8, AssetHandle<MaterialAsset>>,
}

impl WorldMaterialAsset {
    async fn load_asset<F: voxrs_types::io::FileSystem>(
        path: &crate::AssetPath,
        manager: &mut crate::AssetManager<F>,
        _device: Option<&wgpu::Device>,
        _queue: Option<&wgpu::Queue>,
    ) -> Result<Self, crate::handle::AssetLoadError> {
        let result;
        if let Ok(s) = F::read_text(path).await {
            result = Ok(WorldMaterialAsset::new(&s, manager));
        } else {
            result = Err(AssetLoadError::Failed);
        }
        result
    }

    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldMaterialAssetRaw = serde_json::from_str(s).unwrap();

        let mut material_handles = HashMap::new();
        for entity in &raw.materials {
            let material = asset_manager.get::<MaterialAsset>(&AssetPath::from(&entity.material));
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
