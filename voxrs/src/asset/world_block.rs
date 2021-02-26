use serde::Deserialize;

use crate::io::FileSystem;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, WorldBlockMaterialAsset,
};

#[derive(Asset)]
pub struct WorldBlockAsset {
    pub world_size: WorldSize,
    pub block_size: f32,
    pub block_material: AssetHandle<WorldBlockMaterialAsset>,
    pub world_chunk: Vec<WorldChunk>, // x, y, z order
}

impl WorldBlockAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldBlockAssetRaw = serde_json::from_str(s).unwrap();
        Self {
            world_size: raw.world_size,
            block_size: raw.block_size,
            block_material: asset_manager.get(&AssetPath::from_str(&raw.block_material)),
            world_chunk: raw.world_chunk,
        }
    }
}

#[derive(Deserialize)]
struct WorldBlockAssetRaw {
    world_size: WorldSize,
    block_size: f32,
    block_material: String,
    world_chunk: Vec<WorldChunk>,
}

/// block count in x, y, z
/// each should be multiple of CHUNK_CUBE_LEN
#[derive(Deserialize)]
#[allow(dead_code)] // todo: remove
pub struct WorldSize {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Deserialize)]
#[allow(dead_code)] // todo: remove
pub struct WorldChunk {
    blocks: Vec<u8>, // CUBE_CHUNK_LEN ^ 3 (== CHUNK_TOTAL_CUBE_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_world_chunk() {
        let s = r#"{ "blocks": [1, 2, 3, 4] }"#;
        let _world_chunk: WorldChunk = serde_json::from_str(s).unwrap();
    }
}
