#![allow(dead_code)] // todo: remove

use serde::Deserialize;

use crate::blueprint::{CHUNK_CUBE_LEN, CHUNK_TOTAL_CUBE_COUNT};
use crate::io::FileSystem;
use crate::math::*;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, WorldBlockMaterialAsset,
};

#[derive(Asset)]
pub struct WorldBlockAsset {
    pub world_size: WorldSize,
    pub block_size: BlockSize,
    pub block_material: AssetHandle<WorldBlockMaterialAsset>,
    pub world_chunk: Vec<WorldChunk>, // x, y, z order
}

impl WorldBlockAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldBlockAssetRaw = serde_json::from_str(s).unwrap();
        raw.validate();

        Self {
            world_size: raw.world_size,
            block_size: raw.block_size,
            block_material: asset_manager.get(&AssetPath::from_str(&raw.block_material)),
            world_chunk: raw.world_chunk,
        }
    }

    pub fn get_world_pos(&self, idx: i32) -> Vector3 {
        chunk_idx_to_world_pos(&self.world_size, self.block_size.to_f32(), idx)
    }
}

fn chunk_idx_to_world_pos(world_size: &WorldSize, block_size: f32, idx: i32) -> Vector3 {
    let chunk_count = (
        (world_size.x as f32 / CHUNK_CUBE_LEN as f32 / block_size) as i32,
        (world_size.y as f32 / CHUNK_CUBE_LEN as f32 / block_size) as i32,
        (world_size.z as f32 / CHUNK_CUBE_LEN as f32 / block_size) as i32,
    );

    let chunk_x = idx % chunk_count.0;
    let chunk_y = (idx / chunk_count.0) % chunk_count.1;
    let chunk_z = idx / (chunk_count.0 * chunk_count.1);

    Vector3::new(
        chunk_x as f32 * block_size * CHUNK_CUBE_LEN as f32,
        chunk_y as f32 * block_size * CHUNK_CUBE_LEN as f32,
        chunk_z as f32 * block_size * CHUNK_CUBE_LEN as f32,
    )
}

#[derive(Deserialize)]
struct WorldBlockAssetRaw {
    world_size: WorldSize,
    block_size: BlockSize,
    block_material: String,
    world_chunk: Vec<WorldChunk>,
}

#[derive(Deserialize)]
pub enum BlockSize {
    Xs, // 0.25
    S, // 0.5
    M, // 1
    L, // 2
    Xl, // 4
}

impl BlockSize {
    pub fn to_f32(&self) -> f32 {
        match self {
            &BlockSize::Xs => 0.25,
            &BlockSize::S => 0.5,
            &BlockSize::M => 1.0,
            &BlockSize::L => 2.0,
            &BlockSize::Xl => 4.0,
        }
    }
}

impl WorldBlockAssetRaw {
    fn validate(&self) {
        // check world size
        let chunk_len = (CHUNK_CUBE_LEN as f32 * self.block_size.to_f32()) as i32;
        
        assert_eq!(self.world_size.x % chunk_len, 0);
        assert_eq!(self.world_size.y % chunk_len, 0);
        assert_eq!(self.world_size.z % chunk_len, 0);

        // check cube counts in chunk
        for chunk in &self.world_chunk {
            assert_eq!(chunk.blocks.len(), CHUNK_TOTAL_CUBE_COUNT);
        }
    }
}

/// block count in x, y, z
/// each should be multiple of CHUNK_CUBE_LEN
#[derive(Deserialize)]
pub struct WorldSize {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldSize {
    pub fn chunk_count(&self) -> (i32, i32, i32) {
        (
            self.x / CHUNK_CUBE_LEN as i32,
            self.y / CHUNK_CUBE_LEN as i32,
            self.z / CHUNK_CUBE_LEN as i32,
        )
    }
}

#[derive(Deserialize)]
pub struct WorldChunk {
    pub idx: i32,        // chunk index (x, y, z order)
    pub blocks: Vec<u8>, // CUBE_CHUNK_LEN ^ 3 (== CHUNK_TOTAL_CUBE_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_world_chunk() {
        let s = r#"{ "idx": 1, "blocks": [1, 2, 3, 4] }"#;
        let world_chunk: WorldChunk = serde_json::from_str(s).unwrap();
        assert_eq!(world_chunk.idx, 1);
    }

    #[test]
    fn test_chunk_idx_to_world_pos() {
        let world_size = WorldSize {
            x: 32,
            y: 64,
            z: 128,
        };
        let block_size = 0.5;

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 0);
        assert_eq!(world_pos, Vector3::new(0.0, 0.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 1);
        assert_eq!(world_pos, Vector3::new(8.0, 0.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 4);
        assert_eq!(world_pos, Vector3::new(0.0, 8.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 31);
        assert_eq!(world_pos, Vector3::new(24.0, 56.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 32);
        assert_eq!(world_pos, Vector3::new(0.0, 0.0, 8.0));

        let world_pos = chunk_idx_to_world_pos(&world_size, block_size, 4 * 8 * 16 - 1);
        assert_eq!(world_pos, Vector3::new(24.0, 56.0, 120.0));
    }
}
